// SPDX-License-Identifier: GPL-3.0-or-later

use super::receiver::{ReceiveFuture, Receiver};
use crate::{
    device::pci::xhci::{
        self,
        structures::{
            descriptor,
            ring::{event::trb::completion::Completion, transfer},
        },
    },
    mem::allocator::page_box::PageBox,
};
use alloc::{sync::Arc, vec::Vec};
use futures_util::task::AtomicWaker;
use spinning_top::Spinlock;
use transfer::trb::{control::DescTyIdx, Trb};
use x86_64::PhysAddr;

pub struct Sender {
    ring: transfer::Ring,
    receiver: Arc<Spinlock<Receiver>>,
    doorbell_writer: DoorbellWriter,
    waker: Arc<Spinlock<AtomicWaker>>,
}
impl Sender {
    pub fn new(receiver: Arc<Spinlock<Receiver>>, doorbell_writer: DoorbellWriter) -> Self {
        Self {
            ring: transfer::Ring::new(),
            receiver,
            doorbell_writer,
            waker: Arc::new(Spinlock::new(AtomicWaker::new())),
        }
    }

    pub fn ring_addr(&self) -> PhysAddr {
        self.ring.phys_addr()
    }

    pub async fn get_device_descriptor(&mut self) -> PageBox<descriptor::Device> {
        let b = PageBox::user(descriptor::Device::default());
        let (setup, data, status) =
            Trb::new_get_descriptor(&b, DescTyIdx::new(descriptor::Ty::Device, 0));
        self.issue_trbs(&[setup, data, status]).await;
        b
    }

    pub async fn get_configuration_descriptor(&mut self) -> PageBox<[u8]> {
        let b = PageBox::user_slice(0, 4096);
        let (setup, data, status) =
            Trb::new_get_descriptor(&b, DescTyIdx::new(descriptor::Ty::Configuration, 0));
        self.issue_trbs(&[setup, data, status]).await;
        debug!("Got TRBs");
        b
    }

    pub async fn issue_normal_trb<T: ?Sized>(&mut self, b: &PageBox<T>) {
        let t = Trb::new_normal(&b);
        debug!("Normal TRB: {:X?}", t);
        self.issue_trbs(&[t]).await;
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
                .lock()
                .add_entry(a, self.waker.clone())
                .expect("Sender is already registered.");
        }
    }

    fn write_to_doorbell(&mut self) {
        self.doorbell_writer.write();
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
    slot_id: u8,
    val: u32,
}
impl DoorbellWriter {
    pub fn new(slot_id: u8, val: u32) -> Self {
        Self { slot_id, val }
    }

    pub fn write(&mut self) {
        xhci::handle_registers(|r| {
            let d = &mut r.doorbell_array;
            d.update(self.slot_id.into(), |d| *d = self.val);
        });
    }
}
