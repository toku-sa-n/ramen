// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::structures::descriptor::Descriptor;
use crate::{
    device::pci::xhci::{
        exchanger::{command, receiver::Receiver, transfer},
        structures::{
            context::{self, Context},
            dcbaa::DeviceContextBaseAddressArray,
            descriptor,
            ring::CycleBit,
        },
    },
    mem::allocator::page_box::PageBox,
};
use alloc::{rc::Rc, vec::Vec};
use bit_field::BitField;
use core::cell::RefCell;
use futures_intrusive::sync::LocalMutex;
use num_traits::FromPrimitive;
use transfer::DoorbellWriter;

use super::Port;

pub struct Slot {
    id: u8,
    sender: transfer::Sender,
    dcbaa: Rc<RefCell<DeviceContextBaseAddressArray>>,
    context: Context,
}
impl Slot {
    pub fn new(port: Port, id: u8, receiver: Rc<RefCell<Receiver>>) -> Self {
        Self {
            id,
            sender: transfer::Sender::new(
                port.transfer_ring,
                receiver,
                DoorbellWriter::new(port.registers, id),
            ),
            dcbaa: port.dcbaa,
            context: port.context,
        }
    }

    pub async fn init_device_slot(&mut self, runner: Rc<LocalMutex<command::Sender>>) {
        self.register_with_dcbaa();
        self.issue_address_device(runner).await;
    }

    pub async fn get_device_descriptor(&mut self) -> PageBox<descriptor::Device> {
        self.sender.get_device_descriptor().await
    }

    pub async fn enable_endpoint(&mut self) {
        let descs = self.get_configuration_descriptors().await;
        for d in descs {
            self.init_context_with_descriptor(&d);
        }
    }

    pub async fn get_configuration_descriptors(&mut self) -> Vec<Descriptor> {
        let r = self.get_raw_configuration_descriptors().await;
        RawDescriptorParser::new(r).parse()
    }

    fn init_context_with_descriptor(&mut self, d: &Descriptor) {
        if let Descriptor::Endpoint(ep) = d {
            let dci = Self::calculate_dci(ep);
            self.context.input.control_mut().set_aflag(dci.into());
            self.init_context(ep);
        }
    }

    fn calculate_dci(ep: &descriptor::Endpoint) -> u8 {
        let a = ep.endpoint_address;
        2 * a.get_bits(0..=3) + a.get_bit(7) as u8
    }

    fn init_context(&mut self, ep: &descriptor::Endpoint) {
        let c = self.get_context(ep);
        c.set_endpoint_type(Self::endpoint_ty(ep));
        c.set_max_packet_size(ep.max_packet_size);
        c.set_max_burst_size(0);
        c.set_dequeue_cycle_state(CycleBit::new(true));
        c.set_max_primary_streams(0);
        c.set_mult(0);
        c.set_error_count(3);
    }

    fn get_context(&mut self, ep: &descriptor::Endpoint) -> &mut context::Endpoint {
        let ep_idx: usize = ep.endpoint_address.get_bits(0..=3).into();
        let out_input = ep.endpoint_address.get_bit(7);
        let context_inout = &mut self.context.output_device.ep_inout[ep_idx];
        if out_input {
            &mut context_inout.input
        } else {
            &mut context_inout.out
        }
    }

    fn endpoint_ty(ep: &descriptor::Endpoint) -> context::EndpointType {
        context::EndpointType::from_u8(if ep.attributes == 0 {
            0
        } else {
            ep.attributes + if ep.endpoint_address == 0 { 0 } else { 4 }
        })
        .unwrap()
    }

    async fn get_raw_configuration_descriptors(&mut self) -> PageBox<[u8]> {
        self.sender.get_configuration_descriptor().await
    }

    fn register_with_dcbaa(&mut self) {
        self.dcbaa.borrow_mut()[self.id.into()] = self.context.output_device.phys_addr();
    }

    async fn issue_address_device(&mut self, runner: Rc<LocalMutex<command::Sender>>) {
        runner
            .lock()
            .await
            .address_device(self.context.input.phys_addr(), self.id)
            .await;
    }
}

struct RawDescriptorParser {
    raw: PageBox<[u8]>,
    current: usize,
    len: usize,
}
impl RawDescriptorParser {
    fn new(raw: PageBox<[u8]>) -> Self {
        let len = raw.len();
        Self {
            raw,
            current: 0,
            len,
        }
    }

    fn parse(&mut self) -> Vec<Descriptor> {
        let mut v = Vec::new();
        while self.current < self.len && self.raw[self.current] > 0 {
            match self.parse_first_descriptor() {
                Ok(t) => v.push(t),
                Err(e) => warn!("Error: {:?}", e),
            }
        }
        v
    }

    fn parse_first_descriptor(&mut self) -> Result<Descriptor, descriptor::Error> {
        let raw = self.cut_raw_descriptor();
        Descriptor::from_slice(&raw)
    }

    fn cut_raw_descriptor(&mut self) -> Vec<u8> {
        let len: usize = self.raw[self.current].into();
        let v = self.raw[self.current..(self.current + len)].to_vec();
        self.current += len;
        v
    }
}
