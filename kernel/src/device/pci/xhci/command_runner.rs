// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::ring::{command, trb::CommandComplete},
    alloc::{collections::BTreeMap, rc::Rc},
    core::{
        cell::RefCell,
        future::Future,
        pin::Pin,
        task::{Context, Poll},
    },
    futures_util::task::AtomicWaker,
    x86_64::PhysAddr,
};

pub struct Runner {
    ring: Rc<RefCell<command::Ring>>,
    receiver: Rc<RefCell<CommandCompletionReceiver>>,
    waker: Rc<RefCell<AtomicWaker>>,
}
impl Runner {
    pub fn new(
        ring: Rc<RefCell<command::Ring>>,
        receiver: Rc<RefCell<CommandCompletionReceiver>>,
    ) -> Self {
        Self {
            ring,
            receiver,
            waker: Rc::new(RefCell::new(AtomicWaker::new())),
        }
    }

    pub async fn enable_device_slot(&mut self) -> Result<u8, command::Error> {
        let addr_to_trb = self.ring.borrow_mut().send_enable_slot()?;
        self.register_to_receiver(addr_to_trb);
        let completion_trb = self.get_trb(addr_to_trb).await;
        Ok(completion_trb.slot_id())
    }

    fn register_to_receiver(&mut self, addr_to_trb: PhysAddr) {
        self.receiver
            .borrow_mut()
            .add_entry(addr_to_trb, self.waker.clone());
    }

    async fn get_trb(&mut self, addr_to_trb: PhysAddr) -> CommandComplete {
        ReceiveFuture::new(addr_to_trb, self.receiver.clone(), self.waker.clone()).await
    }
}

pub struct CommandCompletionReceiver {
    trbs: BTreeMap<PhysAddr, Option<CommandComplete>>,
    wakers: BTreeMap<PhysAddr, Rc<RefCell<AtomicWaker>>>,
}
impl CommandCompletionReceiver {
    pub fn new() -> Self {
        Self {
            trbs: BTreeMap::new(),
            wakers: BTreeMap::new(),
        }
    }

    pub fn add_entry(&mut self, addr_to_trb: PhysAddr, waker: Rc<RefCell<AtomicWaker>>) {
        self.insert_entry(addr_to_trb, waker).unwrap()
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

    pub fn receive(&mut self, trb: CommandComplete) {
        if let Err(e) = self.insert_trb_and_wake_runner(trb) {
            panic!("Failed to receive a command completion trb: {:?}", e);
        }
    }

    fn insert_trb_and_wake_runner(&mut self, trb: CommandComplete) -> Result<(), Error> {
        let addr_to_trb = PhysAddr::new(trb.addr_to_command_trb());
        self.insert_trb(trb)?;
        self.wake_runner(addr_to_trb)?;
        Ok(())
    }

    fn insert_trb(&mut self, trb: CommandComplete) -> Result<(), Error> {
        let addr_to_trb = PhysAddr::new(trb.addr_to_command_trb());
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

    fn remove_entry(&mut self, addr_to_trb: PhysAddr) -> Option<CommandComplete> {
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
    receiver: Rc<RefCell<CommandCompletionReceiver>>,
    waker: Rc<RefCell<AtomicWaker>>,
}
impl ReceiveFuture {
    fn new(
        addr_to_trb: PhysAddr,
        receiver: Rc<RefCell<CommandCompletionReceiver>>,
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
    type Output = CommandComplete;

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
