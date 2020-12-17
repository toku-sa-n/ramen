// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
    super::structures::ring::{
        command::{self, trb::Trb},
        event::trb::completion::Completion,
    },
    receiver::{ReceiveFuture, Receiver},
};
use alloc::sync::Arc;
use futures_util::task::AtomicWaker;
use spinning_top::Spinlock;
use x86_64::PhysAddr;

pub struct Sender {
    ring: Arc<Spinlock<command::Ring>>,
    receiver: Arc<Spinlock<Receiver>>,
    waker: Arc<Spinlock<AtomicWaker>>,
}
impl Sender {
    pub fn new(ring: Arc<Spinlock<command::Ring>>, receiver: Arc<Spinlock<Receiver>>) -> Self {
        Self {
            ring,
            receiver,
            waker: Arc::new(Spinlock::new(AtomicWaker::new())),
        }
    }

    pub async fn noop(&mut self) {
        let t = Trb::new_noop();
        self.issue_trb(t).await;
        info!("NOOP SUCCEESS");
    }

    pub async fn enable_device_slot(&mut self) -> u8 {
        let t = Trb::new_enable_slot();
        self.issue_trb(t).await.slot_id()
    }

    pub async fn address_device(&mut self, input_context_addr: PhysAddr, slot_id: u8) {
        let t = Trb::new_address_device(input_context_addr, slot_id);
        self.issue_trb(t).await;
    }

    pub async fn configure_endpoint(&mut self, context_addr: PhysAddr, slot_id: u8) {
        let t = Trb::new_configure_endpoint(context_addr, slot_id);
        self.issue_trb(t).await;
    }

    async fn issue_trb(&mut self, t: Trb) -> Completion {
        let a = self.ring.lock().enqueue(t);
        self.register_with_receiver(a);
        self.get_trb(a).await
    }

    fn register_with_receiver(&mut self, addr_to_trb: PhysAddr) {
        self.receiver
            .lock()
            .add_entry(addr_to_trb, self.waker.clone())
            .expect("Sender is already registered.");
    }

    async fn get_trb(&mut self, addr_to_trb: PhysAddr) -> Completion {
        ReceiveFuture::new(addr_to_trb, self.receiver.clone(), self.waker.clone()).await
    }
}
