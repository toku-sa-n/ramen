// SPDX-License-Identifier: GPL-3.0-or-later

mod exchanger;
mod port;
mod structures;
mod xhc;

use super::config::bar;
use crate::multitask::{self, task::Task};
use alloc::sync::Arc;
use conquer_once::spin::OnceCell;
use spinning_top::Spinlock;
use structures::{
    dcbaa,
    registers::Registers,
    ring::{command, event},
    scratchpad,
};

static REGISTERS: OnceCell<Spinlock<Registers>> = OnceCell::uninit();

pub async fn task() {
    if init_registers().is_err() {
        warn!("xHC not found.");
        return;
    }

    let event_ring = init();

    port::spawn_all_connected_port_tasks();

    multitask::add(Task::new_poll(event::task(event_ring)));

    info!("Issuing the NOOP trb.");
    exchanger::command::noop().await;
}

fn init_registers() -> Result<(), XhcNotFound> {
    match iter_devices().next() {
        Some(r) => {
            REGISTERS.init_once(|| Spinlock::new(r));
            Ok(())
        }
        None => Err(XhcNotFound),
    }
}

#[derive(Debug)]
struct XhcNotFound;

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

fn init() -> event::Ring {
    let mut event_ring = event::Ring::new();
    let command_ring = Arc::new(Spinlock::new(command::Ring::new()));

    xhc::init();

    event_ring.init();
    command_ring.lock().init();
    dcbaa::init();
    scratchpad::init();
    exchanger::command::init(command_ring);

    xhc::run();
    xhc::ensure_no_error_occurs();

    event_ring
}

fn iter_devices() -> impl Iterator<Item = Registers> {
    super::iter_devices().filter_map(|device| {
        if device.is_xhci() {
            // SAFETY: This operation is safe because MMIO base address is generated from the 0th
            // BAR.
            Some(unsafe { Registers::new(device.base_address(bar::Index::new(0))) })
        } else {
            None
        }
    })
}
