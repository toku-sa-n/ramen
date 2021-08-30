// SPDX-License-Identifier: GPL-3.0-or-later

use crate::process;
use alloc::collections::VecDeque;
use conquer_once::spin::Lazy;
use core::sync::atomic::{AtomicI32, Ordering};
use spinning_top::{Spinlock, SpinlockGuard};

static WOKEN_PIDS: Lazy<Spinlock<VecDeque<process::SlotId>>> =
    Lazy::new(|| Spinlock::new(VecDeque::new()));

pub(crate) static CURRENT: AtomicI32 = AtomicI32::new(0);

pub(in crate::process) fn add(id: process::SlotId) {
    lock_queue().push_back(id);
}

pub(in crate::process) fn active_pid() -> process::SlotId {
    CURRENT.load(Ordering::Relaxed)
}

pub(in crate::process) fn pop() -> process::SlotId {
    lock_queue()
        .pop_front()
        .expect("All processes are terminated.")
}

pub(in crate::process) fn push(id: process::SlotId) {
    lock_queue().push_back(id);
}

pub(in crate::process) fn next() -> process::SlotId {
    lock_queue()[0]
}

fn lock_queue() -> SpinlockGuard<'static, VecDeque<process::SlotId>> {
    WOKEN_PIDS
        .try_lock()
        .expect("Failed to acquire the lock of `WOKEN_PIDS`.")
}
