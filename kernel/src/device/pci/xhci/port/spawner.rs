// SPDX-License-Identifier: GPL-3.0-or-later

use super::resetter::Resetter;
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
    let p = Resetter::new(port_idx);
    if spawnable(&p) {
        spawn(p);
        Ok(())
    } else {
        Err(PortNotConnected)
    }
}

fn spawn(p: Resetter) {
    mark_as_spawned(&p);
    add_task_for_port(p);
}

fn add_task_for_port(p: Resetter) {
    multitask::add(Task::new(super::main(p)));
}

fn spawnable(p: &Resetter) -> bool {
    p.connected() && !spawned(p)
}

fn spawned(p: &Resetter) -> bool {
    SPAWN_STATUS.lock()[usize::from(p.port_number())]
}

fn mark_as_spawned(p: &Resetter) {
    SPAWN_STATUS.lock()[usize::from(p.port_number())] = true;
}

#[derive(Debug)]
pub struct PortNotConnected;
