// SPDX-License-Identifier: GPL-3.0-or-later

use super::structures::registers;
use crate::multitask::{self, task::Task};
use alloc::collections::VecDeque;
use conquer_once::spin::Lazy;
use core::{future::Future, pin::Pin, task::Poll};
use futures_util::task::AtomicWaker;
use not_reset::NotReset;
use slot_assigned::SlotAssigned;
use slot_not_assigned::SlotNotAssigned;
use spinning_top::Spinlock;

mod class_driver;
mod endpoint;
mod not_reset;
mod slot_assigned;
mod slot_not_assigned;
mod spawner;

static CURRENT_RESET_PORT: Lazy<Spinlock<ResetPort>> =
    Lazy::new(|| Spinlock::new(ResetPort::new()));

struct ResetPort {
    resetting: bool,
    wakers: VecDeque<AtomicWaker>,
}
impl ResetPort {
    fn new() -> Self {
        Self {
            resetting: false,
            wakers: VecDeque::new(),
        }
    }

    fn complete_reset(&mut self) {
        self.resetting = false;
        if let Some(w) = self.wakers.pop_front() {
            w.wake();
        }
    }

    fn resettable(&mut self, waker: AtomicWaker) -> bool {
        if self.resetting {
            self.wakers.push_back(waker);
            false
        } else {
            self.resetting = true;
            true
        }
    }
}

pub fn try_spawn(port_idx: u8) -> Result<(), spawner::PortNotConnected> {
    spawner::try_spawn(port_idx)
}

async fn main(port: NotReset) {
    let mut eps = init_port_and_slot_exclusively(port).await;
    eps.init().await;

    match eps.ty() {
        (3, 1, 2) => {
            multitask::add(Task::new_poll(class_driver::mouse::task(eps)));
        }
        (3, 1, 1) => {
            multitask::add(Task::new_poll(class_driver::keyboard::task(eps)));
        }
        (8, _, _) => multitask::add(Task::new(class_driver::mass_storage::task(eps))),
        t => warn!("Unknown device: {:?}", t),
    }
}

async fn init_port_and_slot_exclusively(port: NotReset) -> endpoint::AddressAssigned {
    let reset_waiter = ResetWaiterFuture;
    reset_waiter.await;

    let port_idx = port.port_number();
    let slot = init_port_and_slot(port).await;
    CURRENT_RESET_PORT.lock().complete_reset();
    info!("Port {} reset completed.", port_idx);
    endpoint::AddressAssigned::new(slot).await
}

async fn init_port_and_slot(p: NotReset) -> SlotAssigned {
    let mut slot_not_assigned = p.reset();
    slot_not_assigned.init_context();

    let mut slot = SlotAssigned::new(slot_not_assigned).await;
    slot.init().await;
    debug!("Slot initialized");
    slot
}

pub fn spawn_all_connected_port_tasks() {
    spawner::spawn_all_connected_ports();
}

fn max_num() -> u8 {
    registers::handle(|r| r.capability.hcsparams1.read().number_of_ports())
}
struct ResetWaiterFuture;
impl Future for ResetWaiterFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
        let waker = AtomicWaker::new();
        waker.register(cx.waker());
        if CURRENT_RESET_PORT.lock().resettable(waker) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
