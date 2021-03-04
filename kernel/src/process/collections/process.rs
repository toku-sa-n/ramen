// SPDX-License-Identifier: GPL-3.0-or-later

use super::woken_pid;
use crate::{process, process::Process};
use alloc::vec::Vec;
use core::ops::DerefMut;
use spinning_top::Spinlock;

static PROCESSES: Spinlock<Vec<Process>> = Spinlock::new(Vec::new());

pub(in crate::process) fn add(p: Process) {
    lock_processes().deref_mut().push(p)
}

pub(in crate::process) fn handle_running_mut<T, U>(f: T) -> U
where
    T: FnOnce(&mut Process) -> U,
{
    let id = woken_pid::active_pid();
    handle_mut(id, f)
}

pub(in crate::process) fn handle_mut<T, U>(id: process::Id, f: T) -> U
where
    T: FnOnce(&mut Process) -> U,
{
    let mut l = lock_processes();
    let mut i = l.deref_mut().iter_mut();
    let p = i.find(|p| p.id == id);
    let p = p.expect("No such process.");

    f(p)
}

pub(in crate::process) fn handle_running<T, U>(f: T) -> U
where
    T: FnOnce(&Process) -> U,
{
    let id = woken_pid::active_pid();
    handle(id, f)
}

pub(in crate::process) fn handle<T, U>(id: process::Id, f: T) -> U
where
    T: FnOnce(&Process) -> U,
{
    let mut l = lock_processes();
    let mut i = l.deref_mut().iter();
    let p = i.find(|p| p.id == id);
    let p = p.expect("No such process.");

    f(p)
}

fn lock_processes() -> impl DerefMut<Target = Vec<Process>> {
    PROCESSES
        .try_lock()
        .expect("Failed to acquire the lock of `PROCESSES`.")
}
