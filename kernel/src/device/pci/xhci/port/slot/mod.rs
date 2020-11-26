// SPDX-License-Identifier: GPL-3.0-or-later

use core::cell::RefCell;

use alloc::rc::Rc;
use futures_intrusive::sync::LocalMutex;
use transfer::DoorbellWriter;

use crate::{
    device::pci::xhci::{
        exchanger::{command, receiver::Receiver, transfer},
        structures::{context::Context, dcbaa::DeviceContextBaseAddressArray, descriptor},
    },
    mem::allocator::page_box::PageBox,
};

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

    pub async fn get_configuration_descriptor(&mut self) -> PageBox<[u8]> {
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
