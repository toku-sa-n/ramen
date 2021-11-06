// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
// A workaround for the `derive_builder` crate.
#![allow(clippy::default_trait_access)]

extern crate alloc;

use {
    alloc::sync::Arc,
    futures_intrusive::sync::{GenericMutex, GenericMutexGuard},
    multitask::{executor::Executor, task::Task},
    pci::config::bar,
    spinning_top::{RawSpinlock, Spinlock},
    structures::{
        dcbaa, extended_capabilities, registers,
        ring::{command, event},
        scratchpad,
    },
    x86_64::PhysAddr,
};

pub(crate) type Futurelock<T> = GenericMutex<RawSpinlock, T>;
pub(crate) type FuturelockGuard<'a, T> = GenericMutexGuard<'a, RawSpinlock, T>;

mod exchanger;
mod multitask;
mod pci;
mod port;
mod structures;
mod xhc;

#[no_mangle]
pub fn main() {
    ralib::init();
    raheap::init();

    init();

    let mut executor = Executor::new();
    executor.run();
}

pub(crate) fn init() {
    if xhc::exists() {
        init_and_spawn_tasks();
    }
}

fn init_statics() {
    let a = iter_xhc().next().expect("xHC does not exist.");

    // SAFETY: BAR 0 address is passed.
    unsafe {
        registers::init(a);
        extended_capabilities::init(a);
    }
}

fn init_and_spawn_tasks() {
    init_statics();

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

    spawn_tasks(event_ring);
}

fn spawn_tasks(e: event::Ring) {
    port::spawn_all_connected_port_tasks();

    multitask::add(Task::new_poll(event::task(e)));
}

fn iter_xhc() -> impl Iterator<Item = PhysAddr> {
    pci::iter_devices().filter_map(|device| {
        if device.is_xhci() {
            Some(device.base_address(bar::Index::new(0)))
        } else {
            None
        }
    })
}
