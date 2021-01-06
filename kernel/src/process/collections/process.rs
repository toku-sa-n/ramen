// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{process, process::Process};
use alloc::collections::BTreeMap;
use spinning_top::{Spinlock, SpinlockGuard};

use super::woken_pid;

static PROCESSES: Spinlock<BTreeMap<process::Id, Process>> = Spinlock::new(BTreeMap::new());

pub(in crate::process) fn add(p: Process) {
    let id = p.id();
    PROCESSES
        .lock()
        .insert(id, p)
        .expect_none("Duplicated process.");
}

pub(in crate::process) fn handle_running<T, U>(f: T) -> U
where
    T: Fn(&Process) -> U,
{
    let id = woken_pid::active_pid();
    let l = lock_processes();
    let p = l
        .get(&id)
        .unwrap_or_else(|| panic!("Process of PID {} does not exist.", id.as_i32()));
    f(p)
}

pub(in crate::process) fn handle_running_mut<T, U>(f: T) -> U
where
    T: Fn(&mut Process) -> U,
{
    let id = woken_pid::active_pid();
    let mut l = lock_processes();
    let p = l
        .get_mut(&id)
        .unwrap_or_else(|| panic!("Process of PID {} does not exist.", id.as_i32()));
    f(p)
}

fn lock_processes() -> SpinlockGuard<'static, BTreeMap<process::Id, Process>> {
    PROCESSES
        .try_lock()
        .expect("Failed to acquire the lock of `PROCESSES`.")
}
