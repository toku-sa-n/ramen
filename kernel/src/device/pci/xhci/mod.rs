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

pub async fn task() {
    if init_statics().is_err() {
        warn!("xHC not found.");
        return;
    }

    let event_ring = init();

    port::spawn_all_connected_port_tasks();

    multitask::add(Task::new_poll(event::task(event_ring)));

    info!("Issuing the NOOP trb.");
    exchanger::command::noop().await;
}

fn init_statics() -> Result<(), XhcNotFound> {
    match iter_devices().next() {
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

fn iter_devices() -> impl Iterator<Item = PhysAddr> {
    super::iter_devices().filter_map(|device| {
        if device.is_xhci() {
            Some(device.base_address(bar::Index::new(0)))
        } else {
            None
        }
    })
}
