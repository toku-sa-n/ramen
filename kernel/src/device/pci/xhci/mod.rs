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
        info!("XHCI");
        init_statics().expect("xHC should exist.");

        let event_ring = init();

        spawn_tasks(event_ring);
    }
}

fn init_statics() -> Result<(), XhcNotFound> {
    match iter_xhc().next() {
        Some(a) => {
            registers::init(a);
            extended_capabilities::init(a);
            Ok(())
        }
        None => Err(XhcNotFound),
    }
}

#[derive(Debug)]
struct XhcNotFound;

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
