// SPDX-License-Identifier: GPL-3.0-or-later

use super::Port;
use crate::multitask;
use alloc::{vec, vec::Vec};
use conquer_once::spin::Lazy;
use multitask::task::Task;
use spinning_top::Spinlock;

static SPAWN_STATUS: Lazy<Spinlock<Vec<bool>>> =
    Lazy::new(|| Spinlock::new(vec![false; super::max_num().into()]));

pub(in crate::device::pci::xhci) fn spawn_all_connected_ports() {
    let n = super::max_num();
    for i in 0..n {
        let _ = try_spawn(i + 1);
    }
}

pub(in crate::device::pci::xhci) fn try_spawn(port_idx: u8) -> Result<(), PortNotConnected> {
    let p = Port::new(port_idx);
    if spawnable(&p) {
        spawn(p);
        Ok(())
    } else {
        Err(PortNotConnected)
    }
}

fn spawn(p: Port) {
    mark_as_spawned(&p);
    add_task_for_port(p);
}

fn add_task_for_port(p: Port) {
    multitask::add(Task::new(super::task(p)));
}

fn spawnable(p: &Port) -> bool {
    p.connected() && !spawned(p)
}

fn spawned(p: &Port) -> bool {
    SPAWN_STATUS.lock()[usize::from(p.index)]
}

fn mark_as_spawned(p: &Port) {
    SPAWN_STATUS.lock()[usize::from(p.index)] = true;
}

#[derive(Debug)]
pub struct PortNotConnected;
