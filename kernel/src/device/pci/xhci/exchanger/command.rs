// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
    super::structures::ring::{command, event::trb::CommandCompletion},
    receiver::{ReceiveFuture, Receiver},
};
use alloc::{collections::BTreeMap, rc::Rc};
use core::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use futures_intrusive::sync::LocalMutex;
use futures_util::task::AtomicWaker;
use x86_64::PhysAddr;

pub fn channel(
    ring: Rc<RefCell<command::Ring>>,
) -> (Rc<LocalMutex<Sender>>, Rc<RefCell<Receiver>>) {
    let r = Rc::new(RefCell::new(Receiver::new()));
    let s = Rc::new(LocalMutex::new(Sender::new(ring, r.clone()), false));
    (s, r)
}

pub struct Sender {
    ring: Rc<RefCell<command::Ring>>,
    receiver: Rc<RefCell<Receiver>>,
    waker: Rc<RefCell<AtomicWaker>>,
}
impl Sender {
    pub async fn enable_device_slot(&mut self) -> Result<u8, command::Error> {
        let addr_to_trb = self.ring.borrow_mut().send_enable_slot()?;
        self.register_to_receiver(addr_to_trb);
        let completion_trb = self.get_trb(addr_to_trb).await;
        Ok(completion_trb.slot_id())
    }

    pub async fn address_device(
        &mut self,
        addr_to_input_context: PhysAddr,
        slot_id: u8,
    ) -> Result<(), command::Error> {
        let addr_to_trb = self
            .ring
            .borrow_mut()
            .send_address_device(addr_to_input_context, slot_id)?;
        self.register_to_receiver(addr_to_trb);
        self.get_trb(addr_to_trb).await;
        Ok(())
    }

    fn new(ring: Rc<RefCell<command::Ring>>, receiver: Rc<RefCell<Receiver>>) -> Self {
        Self {
            ring,
            receiver,
            waker: Rc::new(RefCell::new(AtomicWaker::new())),
        }
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
