// SPDX-License-Identifier: GPL-3.0-or-later

use crate::device::pci::xhci::structures::ring::event::trb::completion::Completion;
use alloc::{collections::BTreeMap, rc::Rc};
use core::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use futures_util::task::AtomicWaker;
use x86_64::PhysAddr;

pub struct Receiver {
    trbs: BTreeMap<PhysAddr, Option<Completion>>,
    wakers: BTreeMap<PhysAddr, Rc<RefCell<AtomicWaker>>>,
}
impl Receiver {
    pub fn new() -> Self {
        Self {
            trbs: BTreeMap::new(),
            wakers: BTreeMap::new(),
        }
    }

    pub fn add_entry(
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

    pub fn receive(&mut self, trb: Completion) {
        if let Err(e) = self.insert_trb_and_wake_runner(trb) {
            panic!("Failed to receive a command completion trb: {:?}", e);
        }
    }

    fn insert_trb_and_wake_runner(&mut self, trb: Completion) -> Result<(), Error> {
        let addr_to_trb = trb.addr();
        self.insert_trb(trb)?;
        self.wake_runner(addr_to_trb)?;
        Ok(())
    }

    fn insert_trb(&mut self, trb: Completion) -> Result<(), Error> {
        let addr_to_trb = trb.addr();
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

    fn remove_entry(&mut self, addr_to_trb: PhysAddr) -> Option<Completion> {
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

pub struct ReceiveFuture {
    addr_to_trb: PhysAddr,
    receiver: Rc<RefCell<Receiver>>,
    waker: Rc<RefCell<AtomicWaker>>,
}
impl ReceiveFuture {
    pub fn new(
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
    type Output = Completion;

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
