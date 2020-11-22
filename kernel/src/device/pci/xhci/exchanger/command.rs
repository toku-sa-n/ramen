// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::structures::ring::{command, event::trb::CommandCompletion};
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
            .add_entry(addr_to_trb, self.waker.clone());
    }

    async fn get_trb(&mut self, addr_to_trb: PhysAddr) -> CommandCompletion {
        ReceiveFuture::new(addr_to_trb, self.receiver.clone(), self.waker.clone()).await
    }
}

pub struct Receiver {
    trbs: BTreeMap<PhysAddr, Option<CommandCompletion>>,
    wakers: BTreeMap<PhysAddr, Rc<RefCell<AtomicWaker>>>,
}
impl Receiver {
    pub fn add_entry(&mut self, addr_to_trb: PhysAddr, waker: Rc<RefCell<AtomicWaker>>) {
        self.insert_entry(addr_to_trb, waker).unwrap()
    }

    pub fn receive(&mut self, trb: CommandCompletion) {
        if let Err(e) = self.insert_trb_and_wake_runner(trb) {
            panic!("Failed to receive a command completion trb: {:?}", e);
        }
    }

    fn insert_entry(
        &mut self,
        addr_to_trb: PhysAddr,
        waker: Rc<RefCell<AtomicWaker>>,
    ) -> Result<(), Error> {
        if self.trbs.insert(addr_to_trb, None).is_some() {
            return Err(Error::AddrAlreadyRegistered);
        }

        if self.wakers.insert(addr_to_trb, waker).is_some() {
            return Err(Error::AddrAlreadyRegistered);
        }
        Ok(())
    }

    fn new() -> Self {
        Self {
            trbs: BTreeMap::new(),
            wakers: BTreeMap::new(),
        }
    }

    fn insert_trb_and_wake_runner(&mut self, trb: CommandCompletion) -> Result<(), Error> {
        let addr_to_trb = PhysAddr::new(trb.trb_addr());
        self.insert_trb(trb)?;
        self.wake_runner(addr_to_trb)?;
        Ok(())
    }

    fn insert_trb(&mut self, trb: CommandCompletion) -> Result<(), Error> {
        let addr_to_trb = PhysAddr::new(trb.trb_addr());
        *self
            .trbs
            .get_mut(&addr_to_trb)
            .ok_or(Error::NoSuchAddress)? = Some(trb);
        Ok(())
    }

    fn wake_runner(&mut self, addr_to_trb: PhysAddr) -> Result<(), Error> {
        self.wakers
            .remove(&addr_to_trb)
            .ok_or(Error::NoSuchAddress)?
            .borrow_mut()
            .wake();
        Ok(())
    }

    fn trb_arrives(&self, addr_to_trb: PhysAddr) -> bool {
        match self.trbs.get(&addr_to_trb) {
            Some(trb) => trb.is_some(),
            None => panic!("No such TRB with the address {:?}", addr_to_trb),
        }
    }

    fn remove_entry(&mut self, addr_to_trb: PhysAddr) -> Option<CommandCompletion> {
        match self.trbs.remove(&addr_to_trb) {
            Some(trb) => trb,
            None => panic!("No such receiver with TRB address: {:?}", addr_to_trb),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    AddrAlreadyRegistered,
    NoSuchAddress,
}

struct ReceiveFuture {
    addr_to_trb: PhysAddr,
    receiver: Rc<RefCell<Receiver>>,
    waker: Rc<RefCell<AtomicWaker>>,
}
impl ReceiveFuture {
    fn new(
        addr_to_trb: PhysAddr,
        receiver: Rc<RefCell<Receiver>>,
        waker: Rc<RefCell<AtomicWaker>>,
    ) -> Self {
        Self {
            addr_to_trb,
            receiver,
            waker,
        }
    }
}
impl Future for ReceiveFuture {
    type Output = CommandCompletion;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let waker = self.waker.clone();
        let addr = self.addr_to_trb;
        let receiver = &mut Pin::into_inner(self).receiver;

        waker.borrow_mut().register(cx.waker());
        if receiver.borrow_mut().trb_arrives(addr) {
            waker.borrow_mut().take();
            let trb = receiver.borrow_mut().remove_entry(addr).unwrap();
            Poll::Ready(trb)
        } else {
            Poll::Pending
        }
    }
}
