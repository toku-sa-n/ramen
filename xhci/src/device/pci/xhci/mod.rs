// SPDX-License-Identifier: GPL-3.0-or-later

mod exchanger;
mod port;
mod structures;
mod xhc;

use super::config::bar;
use crate::multitask::{self, task::Task};
use alloc::sync::Arc;
use spinning_top::Spinlock;
use structures::{
    dcbaa, extended_capabilities, registers,
    ring::{command, event},
    scratchpad,
};
use x86_64::PhysAddr;

pub(crate) async fn task() {
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
    super::iter_devices().filter_map(|device| {
        if device.is_xhci() {
            Some(device.base_address(bar::Index::new(0)))
        } else {
            None
        }
    })
}
