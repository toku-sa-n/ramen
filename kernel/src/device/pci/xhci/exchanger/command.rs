// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
    super::structures::ring::{
        command::{self, trb::Trb},
        event::trb::CommandCompletion,
    },
    receiver::{ReceiveFuture, Receiver},
};
use alloc::rc::Rc;
use core::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use futures_util::task::AtomicWaker;
use x86_64::PhysAddr;

pub struct Sender {
    ring: Rc<RefCell<command::Ring>>,
    receiver: Rc<RefCell<Receiver>>,
    waker: Rc<RefCell<AtomicWaker>>,
}
impl Sender {
    pub fn new(ring: Rc<RefCell<command::Ring>>, receiver: Rc<RefCell<Receiver>>) -> Self {
        Self {
            ring,
            receiver,
            waker: Rc::new(RefCell::new(AtomicWaker::new())),
        }
    }

    pub async fn enable_device_slot(&mut self) -> Result<u8, command::Error> {
        let t = Trb::new_enable_slot();
        let addr_to_trb = self.ring.borrow_mut().try_enqueue(t)?;
        self.register_to_receiver(addr_to_trb);
        let completion_trb = self.get_trb(addr_to_trb).await;
        Ok(completion_trb.slot_id())
    }

    pub async fn address_device(
        &mut self,
        input_context_addr: PhysAddr,
        slot_id: u8,
    ) -> Result<(), command::Error> {
        let t = Trb::new_address_device(input_context_addr, slot_id);
        let a = self.ring.borrow_mut().try_enqueue(t)?;
        self.register_to_receiver(a);
        self.get_trb(a).await;
        Ok(())
    }

    fn register_to_receiver(&mut self, addr_to_trb: PhysAddr) {
        self.receiver
            .borrow_mut()
            .add_entry(addr_to_trb, self.waker.clone())
            .expect("Sender is already registered.");
    }

    async fn get_trb(&mut self, addr_to_trb: PhysAddr) -> CommandCompletion {
        ReceiveFuture::new(addr_to_trb, self.receiver.clone(), self.waker.clone()).await
    }
}
