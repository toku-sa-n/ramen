// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;

use super::{descriptor_fetcher::DescriptorFetcher, fully_operational::FullyOperational};
use crate::device::pci::xhci::{
    exchanger,
    exchanger::transfer,
    port::endpoint,
    structures::{context::Context, descriptor, descriptor::Descriptor},
};
use alloc::{sync::Arc, vec::Vec};
use bit_field::BitField;
use spinning_top::Spinlock;
use transfer::DoorbellWriter;
use x86_64::PhysAddr;
use xhci::context::{EndpointHandler, EndpointType};

pub(super) struct EndpointsInitializer {
    cx: Arc<Spinlock<Context>>,
    descriptors: Vec<Descriptor>,
    endpoints: Vec<endpoint::NonDefault>,
    ep0: endpoint::Default,
    slot_number: u8,
}
impl EndpointsInitializer {
    pub(super) fn new(f: DescriptorFetcher, descriptors: Vec<Descriptor>) -> Self {
        let cx = f.context();
        let endpoints = descriptors_to_endpoints(&f, &descriptors);
        let slot_number = f.slot_number();
        let ep0 = f.ep0();

        Self {
            cx,
            descriptors,
            endpoints,
            ep0,
            slot_number,
        }
    }

    pub(super) async fn init(mut self) -> FullyOperational {
        self.init_contexts();
        self.set_context_entries();
        self.configure_endpoint().await;
        FullyOperational::new(self)
    }

    pub(super) fn descriptors(&self) -> Vec<Descriptor> {
        self.descriptors.clone()
    }

    pub(super) fn endpoints(self) -> (endpoint::Default, Vec<endpoint::NonDefault>) {
        (self.ep0, self.endpoints)
    }

    fn init_contexts(&mut self) {
        for e in &mut self.endpoints {
            ContextInitializer::new(&e.descriptor(), &mut self.cx.lock(), e.transfer_ring_addr())
                .init()
        }
    }

    fn set_context_entries(&mut self) {
        let mut cx = self.cx.lock();
        cx.input.device_mut().slot_mut().set_context_entries(31);
    }

    async fn configure_endpoint(&mut self) {
        let a = self.cx.lock().input.phys_addr();
        exchanger::command::configure_endpoint(a, self.slot_number).await;
    }
}

struct ContextInitializer<'a> {
    ep: &'a descriptor::Endpoint,
    cx: &'a mut Context,
    transfer_ring_addr: PhysAddr,
}
impl<'a> ContextInitializer<'a> {
    fn new(
        ep: &'a descriptor::Endpoint,
        context: &'a mut Context,
        transfer_ring: PhysAddr,
    ) -> Self {
        Self {
            ep,
            cx: context,
            transfer_ring_addr: transfer_ring,
        }
    }

    fn init(&mut self) {
        self.set_aflag();
        self.init_ep_context();
    }

    fn set_aflag(&mut self) {
        let dci: usize = self.calculate_dci().into();
        let c = self.cx.input.control_mut();

        c.set_aflag(0);
        c.clear_aflag(1); // See xHCI dev manual 4.6.6.
        c.set_aflag(dci);
    }

    fn calculate_dci(&self) -> u8 {
        let a = self.ep.endpoint_address;
        2 * a.get_bits(0..=3) + a.get_bit(7) as u8
    }

    fn init_ep_context(&mut self) {
        let ep_ty = self.ep.ty();

        self.cx().set_endpoint_type(ep_ty);

        // TODO: This initializes the context only for USB2. Branch if the version of a device is
        // USB3.
        match ep_ty {
            EndpointType::Control => self.init_for_control(),
            EndpointType::BulkOut | EndpointType::BulkIn => self.init_for_bulk(),
            EndpointType::IsochronousOut
            | EndpointType::IsochronousIn
            | EndpointType::InterruptOut
            | EndpointType::InterruptIn => self.init_for_isoch_or_interrupt(),
            EndpointType::NotValid => unreachable!("Not Valid Endpoint should not exist."),
        }
    }

    fn init_for_control(&mut self) {
        assert_eq!(
            self.ep.ty(),
            EndpointType::Control,
            "Not the Control Endpoint."
        );

        let sz = self.ep.max_packet_size;
        let a = self.transfer_ring_addr;
        let c = self.cx();

        c.set_max_packet_size(sz);
        c.set_error_count(3);
        c.set_transfer_ring_dequeue_pointer(a.as_u64());
        c.set_dequeue_cycle_state(true);
    }

    fn init_for_bulk(&mut self) {
        assert!(self.is_bulk(), "Not the Bulk Endpoint.");

        let sz = self.ep.max_packet_size;
        let a = self.transfer_ring_addr;
        let c = self.cx();

        c.set_max_packet_size(sz);
        c.set_max_burst_size(0);
        c.set_error_count(3);
        c.set_max_primary_streams(0);
        c.set_transfer_ring_dequeue_pointer(a.as_u64());
        c.set_dequeue_cycle_state(true);
    }

    fn is_bulk(&self) -> bool {
        let t = self.ep.ty();

        [EndpointType::BulkOut, EndpointType::BulkIn].contains(&t)
    }

    fn init_for_isoch_or_interrupt(&mut self) {
        let t = self.ep.ty();
        assert!(
            self.is_isoch_or_interrupt(),
            "Not the Isochronous or the Interrupt Endpoint."
        );

        let sz = self.ep.max_packet_size;
        let a = self.transfer_ring_addr;
        let c = self.cx();

        c.set_max_packet_size(sz & 0x7ff);
        c.set_max_burst_size(((sz & 0x1800) >> 11).try_into().unwrap());
        c.set_mult(0);

        if let EndpointType::IsochronousOut | EndpointType::IsochronousIn = t {
            c.set_error_count(0);
        } else {
            c.set_error_count(3);
        }
        c.set_transfer_ring_dequeue_pointer(a.as_u64());
        c.set_dequeue_cycle_state(true);
    }

    fn is_isoch_or_interrupt(&self) -> bool {
        let t = self.ep.ty();
        [
            EndpointType::IsochronousOut,
            EndpointType::IsochronousIn,
            EndpointType::InterruptOut,
            EndpointType::InterruptIn,
        ]
        .contains(&t)
    }

    fn cx(&mut self) -> &mut dyn EndpointHandler {
        let ep_i: usize = self.ep.endpoint_address.get_bits(0..=3).into();
        let is_input = self.ep.endpoint_address.get_bit(7);
        let context_inout = self.cx.input.device_mut().endpoints_mut(ep_i);
        if is_input {
            context_inout.input_mut()
        } else {
            context_inout.output_mut()
        }
    }
}

fn descriptors_to_endpoints(
    f: &DescriptorFetcher,
    descriptors: &[Descriptor],
) -> Vec<endpoint::NonDefault> {
    descriptors
        .iter()
        .filter_map(|desc| {
            if let Descriptor::Endpoint(e) = desc {
                let d = DoorbellWriter::new(f.slot_number(), e.doorbell_value());
                let s = transfer::Sender::new(d);
                Some(endpoint::NonDefault::new(*e, s))
            } else {
                None
            }
        })
        .collect()
}
