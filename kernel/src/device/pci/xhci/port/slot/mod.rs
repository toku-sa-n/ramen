// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::structures::descriptor::Descriptor;
use crate::{
    device::pci::xhci::{
        exchanger::{command, receiver::Receiver, transfer},
        structures::{context::Context, dcbaa::DeviceContextBaseAddressArray, descriptor},
    },
    mem::allocator::page_box::PageBox,
};
use alloc::{rc::Rc, vec::Vec};
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

    pub async fn get_configuration_descriptors(&mut self) -> Vec<Descriptor> {
        let r = self.get_raw_configuration_descriptors().await;
        RawDescriptorParser::new(r).parse()
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
        self.len += len;
        v
    }
}
