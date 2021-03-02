// SPDX-License-Identifier: GPL-3.0-or-later

use crate::device::pci::xhci::{
    exchanger::transfer,
    structures::{context::Context, descriptor},
};
use alloc::sync::Arc;
use bit_field::BitField;
use core::convert::TryInto;
use page_box::PageBox;
use spinning_top::Spinlock;
use x86_64::PhysAddr;
use xhci::context::{EndpointHandler, EndpointType};

pub(super) struct Default {
    sender: transfer::Sender,
}
impl Default {
    pub(super) fn new(sender: transfer::Sender) -> Self {
        Self { sender }
    }

    pub(super) fn ring_addr(&self) -> PhysAddr {
        self.sender.ring_addr()
    }

    pub(super) async fn get_max_packet_size(&mut self) -> u16 {
        self.sender
            .get_max_packet_size_from_device_descriptor()
            .await
    }

    pub(super) async fn get_raw_configuration_descriptors(&mut self) -> PageBox<[u8]> {
        self.sender.get_configuration_descriptor().await
    }

    pub(super) async fn set_configuration(&mut self, config_val: u8) {
        self.sender.set_configure(config_val).await;
    }

    pub(super) async fn set_idle(&mut self) {
        self.sender.set_idle().await;
    }

    pub(super) async fn set_boot_protocol(&mut self) {
        self.sender.set_boot_protocol().await;
    }
}

pub(super) struct NonDefault {
    desc: descriptor::Endpoint,
    cx: Arc<Spinlock<Context>>,
    sender: transfer::Sender,
}
impl NonDefault {
    pub(super) fn new(
        desc: descriptor::Endpoint,
        cx: Arc<Spinlock<Context>>,
        sender: transfer::Sender,
    ) -> Self {
        Self { desc, cx, sender }
    }

    pub(super) fn init_context(&mut self) {
        ContextInitializer::new(&self.desc, &mut self.cx.lock(), &self.sender).init();
    }

    pub(super) fn ty(&self) -> EndpointType {
        self.desc.ty()
    }

    pub(super) async fn issue_normal_trb<T: ?Sized>(&mut self, b: &PageBox<T>) {
        self.sender.issue_normal_trb(b).await
    }
}

struct ContextInitializer<'a> {
    ep: &'a descriptor::Endpoint,
    context: &'a mut Context,
    sender: &'a transfer::Sender,
}
impl<'a> ContextInitializer<'a> {
    fn new(
        ep: &'a descriptor::Endpoint,
        context: &'a mut Context,
        sender: &'a transfer::Sender,
    ) -> Self {
        Self {
            ep,
            context,
            sender,
        }
    }

    fn init(&mut self) {
        self.set_aflag();
        self.init_ep_context();
    }

    fn set_aflag(&mut self) {
        let dci: usize = self.calculate_dci().into();
        let c = self.context.input.control_mut();

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
        let a = self.sender.ring_addr();
        let c = self.cx();

        c.set_max_packet_size(sz);
        c.set_error_count(3);
        c.set_transfer_ring_dequeue_pointer(a.as_u64());
        c.set_dequeue_cycle_state(true);
    }

    fn init_for_bulk(&mut self) {
        assert!(self.is_bulk(), "Not the Bulk Endpoint.");

        let sz = self.ep.max_packet_size;
        let a = self.sender.ring_addr();
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
        let a = self.sender.ring_addr();
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
        let context_inout = self.context.input.device_mut().endpoints_mut(ep_i);
        if is_input {
            context_inout.input_mut()
        } else {
            context_inout.output_mut()
        }
    }
}

#[derive(Debug)]
pub(in crate::device::pci::xhci) enum Error {
    NoSuchEndpoint(EndpointType),
}
