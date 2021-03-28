// SPDX-License-Identifier: GPL-3.0-or-later

use super::woken_pid;
use crate::{process, process::Process};
use alloc::collections::BTreeMap;
use spinning_top::{Spinlock, SpinlockGuard};

static PROCESSES: Spinlock<BTreeMap<process::SlotId, Process>> = Spinlock::new(BTreeMap::new());

pub(in crate::process) fn add(p: Process) {
    let id = p.id();
    let r = PROCESSES.lock().insert(id, p);
    assert!(r.is_none(), "Duplicated process.");
}

pub(in crate::process) fn handle_running_mut<T, U>(f: T) -> U
where
    T: FnOnce(&mut Process) -> U,
{
    let id = woken_pid::active_pid();
    handle_mut(id, f)
}

pub(in crate::process) fn handle_mut<T, U>(id: process::SlotId, f: T) -> U
where
    T: FnOnce(&mut Process) -> U,
{
    let mut l = lock_processes();
    let p = l
        .get_mut(&id)
        .unwrap_or_else(|| panic!("Process of PID {} does not exist.", id));
    f(p)
}

pub(in crate::process) fn handle_running<T, U>(f: T) -> U
where
    T: FnOnce(&Process) -> U,
{
    let id = woken_pid::active_pid();
    handle(id, f)
}

pub(in crate::process) fn handle<T, U>(id: process::SlotId, f: T) -> U
where
    T: FnOnce(&Process) -> U,
{
    let l = lock_processes();
    let p = l
        .get(&id)
        .unwrap_or_else(|| panic!("Process of PID {} does not exist.", id));
    f(p)
}

fn lock_processes() -> SpinlockGuard<'static, BTreeMap<process::SlotId, Process>> {
    PROCESSES
        .try_lock()
        .expect("Failed to acquire the lock of `PROCESSES`.")
}
