// SPDX-License-Identifier: GPL-3.0-or-later

use super::receiver::{ReceiveFuture, Receiver};
use crate::{
    device::pci::xhci::structures::{
        descriptor,
        registers::Registers,
        ring::{event::trb::completion::Completion, transfer},
    },
    mem::allocator::page_box::PageBox,
};
use alloc::{rc::Rc, vec::Vec};
use core::cell::RefCell;
use futures_util::task::AtomicWaker;
use transfer::trb::{
    control::{DescTy, DescTyIdx},
    Trb,
};
use x86_64::PhysAddr;

pub struct Sender {
    ring: transfer::Ring,
    receiver: Rc<RefCell<Receiver>>,
    doorbell_writer: DoorbellWriter,
    waker: Rc<RefCell<AtomicWaker>>,
}
impl Sender {
    pub fn new(
        ring: transfer::Ring,
        receiver: Rc<RefCell<Receiver>>,
        doorbell_writer: DoorbellWriter,
    ) -> Self {
        Self {
            ring,
            receiver,
            doorbell_writer,
            waker: Rc::new(RefCell::new(AtomicWaker::new())),
        }
    }

    pub async fn get_device_descriptor(&mut self) -> PageBox<descriptor::Device> {
        let b = PageBox::new(descriptor::Device::default());
        let (setup, data, status) = Trb::new_get_descriptor(&b, DescTyIdx::new(DescTy::Device, 0));
        self.issue_trbs(&[setup, data, status]).await;
        b
    }

    pub async fn get_configuration_descriptor(&mut self) -> PageBox<[u8]> {
        let b = PageBox::new_slice(0, 256);
        let (setup, data, status) =
            Trb::new_get_descriptor(&b, DescTyIdx::new(DescTy::Configuration, 0));
        self.issue_trbs(&[setup, data, status]).await;
        b
    }

    async fn issue_trbs(&mut self, ts: &[Trb]) -> Vec<Option<Completion>> {
        let addrs = self.ring.enqueue(ts);
        self.register_with_receiver(ts, &addrs);
        self.write_to_doorbell();
        self.get_trb(ts, &addrs).await
    }

    fn register_with_receiver(&mut self, ts: &[Trb], addrs: &[PhysAddr]) {
        for (t, addr) in ts.iter().zip(addrs) {
            self.register_trb(t, *addr);
        }
    }

    fn register_trb(&mut self, t: &Trb, a: PhysAddr) {
        if t.ioc() {
            self.receiver
                .borrow_mut()
                .add_entry(a, self.waker.clone())
                .expect("Sender is already registered.");
        }
    }

    fn write_to_doorbell(&mut self) {
        self.doorbell_writer.write(1);
    }

    async fn get_trb(&mut self, ts: &[Trb], addrs: &[PhysAddr]) -> Vec<Option<Completion>> {
        let mut v = Vec::new();
        for (t, a) in ts.iter().zip(addrs) {
            v.push(self.get_single_trb(t, *a).await);
        }
        v
    }

    async fn get_single_trb(&mut self, t: &Trb, addr: PhysAddr) -> Option<Completion> {
        if t.ioc() {
            Some(ReceiveFuture::new(addr, self.receiver.clone(), self.waker.clone()).await)
        } else {
            None
        }
    }
}

pub struct DoorbellWriter {
    registers: Rc<RefCell<Registers>>,
    slot_id: u8,
}
impl DoorbellWriter {
    pub fn new(registers: Rc<RefCell<Registers>>, slot_id: u8) -> Self {
        Self { registers, slot_id }
    }

    pub fn write(&mut self, x: u32) {
        let d = &mut self.registers.borrow_mut().doorbell_array;
        d.update(self.slot_id.into(), |d| *d = x);
    }
}
