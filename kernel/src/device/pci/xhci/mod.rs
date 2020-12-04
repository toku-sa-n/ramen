// SPDX-License-Identifier: GPL-3.0-or-later

mod exchanger;
mod port;
mod structures;
mod xhc;

use super::config::bar;
use crate::{
    multitask::{self, task::Task},
    Futurelock,
};
use alloc::sync::Arc;
use conquer_once::spin::OnceCell;
use exchanger::{command::Sender, receiver::Receiver};
use spinning_top::Spinlock;
use structures::{
    dcbaa::DeviceContextBaseAddressArray,
    registers::Registers,
    ring::{command, event},
};
use xhc::Xhc;

static REGISTERS: OnceCell<Spinlock<Registers>> = OnceCell::uninit();

pub async fn task() {
    init_registers();
    let (event_ring, dcbaa, runner, command_completion_receiver) = init();

    port::spawn_tasks(&runner, &dcbaa, &command_completion_receiver);

    multitask::add(Task::new_poll(event::task(
        event_ring,
        command_completion_receiver,
    )));
}

fn init_registers() {
    REGISTERS.init_once(|| Spinlock::new(iter_devices().next().unwrap()));
}

/// Handle xHCI registers.
///
/// To avoid deadlocking, this method takes a closure. Caller is supposed not to call this method
/// inside the closure, otherwise a deadlock will happen.
///
/// Alternative implementation is to define a method which returns `impl Deref<Target =
/// Registers>`, but this will expand the scope of the mutex guard, increasing the possibility of
/// deadlocks.
fn handle_registers<T, U>(f: T) -> U
where
    T: Fn(&mut Registers) -> U,
{
    let mut r = REGISTERS.try_get().unwrap().lock();
    f(&mut r)
}

// FIXME
#[allow(clippy::type_complexity)]
fn init() -> (
    event::Ring,
    Arc<Spinlock<DeviceContextBaseAddressArray>>,
    Arc<Futurelock<Sender>>,
    Arc<Spinlock<Receiver>>,
) {
    let mut xhc = Xhc::new();
    let mut event_ring = event::Ring::new();
    let command_ring = Arc::new(Spinlock::new(command::Ring::new()));
    let dcbaa = Arc::new(Spinlock::new(DeviceContextBaseAddressArray::new()));
    let receiver = Arc::new(Spinlock::new(Receiver::new()));
    let sender = Arc::new(Futurelock::new(
        Sender::new(command_ring.clone(), receiver.clone()),
        false,
    ));

    xhc.init();

    event_ring.init();
    command_ring.lock().init();
    dcbaa.lock().init();

    xhc.run();

    (event_ring, dcbaa, sender, receiver)
}

fn iter_devices() -> impl Iterator<Item = Registers> {
    super::iter_devices().filter_map(|device| {
        if device.is_xhci() {
            // Safety: This operation is safe because MMIO base address is generated from the 0th
            // BAR.
            Some(unsafe { Registers::new(device.base_address(bar::Index::new(0))) })
        } else {
            None
        }
    })
}
