// SPDX-License-Identifier: GPL-3.0-or-later

use core::{future::Future, pin::Pin, task::Poll};

use super::{
    exchanger::{command, receiver::Receiver},
    structures::{context::Context, registers::operational::PortRegisters},
};
use crate::{
    multitask::{self, task::Task},
    Futurelock,
};
use alloc::{collections::VecDeque, sync::Arc};
use conquer_once::spin::Lazy;
use futures_util::task::AtomicWaker;
use resetter::Resetter;
use slot::Slot;
use spinning_top::Spinlock;

mod class_driver;
mod context;
mod endpoint;
mod resetter;
mod slot;
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

async fn task(
    port: Port,
    runner: Arc<Futurelock<command::Sender>>,
    receiver: Arc<Spinlock<Receiver>>,
) {
    let mut eps = init_port_and_slot(port, runner, receiver).await;
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

async fn init_port_and_slot(
    mut port: Port,
    runner: Arc<Futurelock<command::Sender>>,
    receiver: Arc<Spinlock<Receiver>>,
) -> endpoint::Collection {
    let reset_waiter = ResetWaiterFuture;
    reset_waiter.await;

    let port_idx = port.index;

    port.reset();
    port.init_context();

    let slot_id = runner.lock().await.enable_device_slot().await;

    let mut slot = Slot::new(port, slot_id, receiver);
    slot.init(runner.clone()).await;
    debug!("Slot initialized");
    CURRENT_RESET_PORT.lock().complete_reset();
    info!("Port {} reset completed.", port_idx);
    endpoint::Collection::new(slot, runner).await
}

pub fn spawn_all_connected_port_tasks(
    sender: Arc<Futurelock<command::Sender>>,
    receiver: Arc<Spinlock<Receiver>>,
) {
    spawner::init(sender, receiver);
    spawner::spawn_all_connected_ports();
}

fn max_num() -> u8 {
    super::handle_registers(|r| {
        let params1 = r.capability.hcs_params_1.read();
        params1.max_ports()
    })
}

pub struct Port {
    index: u8,
    context: Context,
}
impl Port {
    fn new(index: u8) -> Self {
        Self {
            index,
            context: Context::new(),
        }
    }

    fn connected(&self) -> bool {
        self.read_port_rg().port_sc.current_connect_status()
    }

    fn reset(&mut self) {
        info!("Resetting port {}", self.index);
        Resetter::new(self.index).reset();
        info!("Port {} is reset.", self.index);
    }

    fn init_context(&mut self) {
        context::Initializer::new(&mut self.context, self.index).init();
    }

    fn read_port_rg(&self) -> PortRegisters {
        super::handle_registers(|r| {
            let port_rg = &r.operational.port_registers;
            port_rg.read((self.index - 1).into())
        })
    }
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
