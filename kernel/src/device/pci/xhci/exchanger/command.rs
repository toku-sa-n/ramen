// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{Futurelock, FuturelockGuard};

use super::{
    super::structures::ring::{
        command::{
            self,
            trb::{AddressDevice, ConfigureEndpoint, EnableSlot, Noop, Trb},
        },
        event::trb::completion::Completion,
    },
    receiver::{self, ReceiveFuture},
};
use alloc::sync::Arc;
use conquer_once::spin::OnceCell;
use futures_util::task::AtomicWaker;
use spinning_top::Spinlock;
use x86_64::PhysAddr;

static SENDER: OnceCell<Futurelock<Sender>> = OnceCell::uninit();

pub(in crate::device::pci::xhci) fn init(r: Arc<Spinlock<command::Ring>>) {
    SENDER
        .try_init_once(|| Futurelock::new(Sender::new(r), true))
        .expect("`Sender` is initialized more than once.")
}

pub(in crate::device::pci::xhci) async fn noop() {
    lock().await.noop().await;
}

pub(in crate::device::pci::xhci) async fn enable_device_slot() -> u8 {
    lock().await.enable_device_slot().await
}

pub(in crate::device::pci::xhci) async fn address_device(input_cx: PhysAddr, slot: u8) {
    lock().await.address_device(input_cx, slot).await;
}

pub(in crate::device::pci::xhci) async fn configure_endpoint(cx: PhysAddr, slot: u8) {
    lock().await.configure_endpoint(cx, slot).await;
}

async fn lock() -> FuturelockGuard<'static, Sender> {
    let s = SENDER.try_get().expect("`SENDER` is not initialized.");
    s.lock().await
}

struct Sender {
    ring: Arc<Spinlock<command::Ring>>,
    waker: Arc<Spinlock<AtomicWaker>>,
}
impl Sender {
    fn new(ring: Arc<Spinlock<command::Ring>>) -> Self {
        Self {
            ring,
            waker: Arc::new(Spinlock::new(AtomicWaker::new())),
        }
    }

    async fn noop(&mut self) {
        let t = Trb::Noop(Noop::default());
        self.issue_trb(t).await;
        info!("NOOP SUCCEESS");
    }

    async fn enable_device_slot(&mut self) -> u8 {
        let t = Trb::EnableSlot(EnableSlot::default());
        self.issue_trb(t).await.slot_id()
    }

    async fn address_device(&mut self, input_context_addr: PhysAddr, slot_id: u8) {
        let t = *AddressDevice::default()
            .set_input_context_ptr(input_context_addr)
            .set_slot_id(slot_id);
        let t = Trb::AddressDevice(t);
        self.issue_trb(t).await;
    }

    async fn configure_endpoint(&mut self, context_addr: PhysAddr, slot_id: u8) {
        let t = *ConfigureEndpoint::default()
            .set_context_addr(context_addr)
            .set_slot_id(slot_id);
        let t = Trb::ConfigureEndpoint(t);
        self.issue_trb(t).await;
    }

    async fn issue_trb(&mut self, t: Trb) -> Completion {
        let a = self.ring.lock().enqueue(t);
        self.register_with_receiver(a);
        self.get_trb(a).await
    }

    fn register_with_receiver(&mut self, addr_to_trb: PhysAddr) {
        receiver::add_entry(addr_to_trb, self.waker.clone())
            .expect("Sender is already registered.");
    }

    async fn get_trb(&mut self, addr_to_trb: PhysAddr) -> Completion {
        ReceiveFuture::new(addr_to_trb, self.waker.clone()).await
    }
}
