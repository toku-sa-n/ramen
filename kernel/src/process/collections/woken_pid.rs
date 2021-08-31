// SPDX-License-Identifier: GPL-3.0-or-later

use crate::process;
use alloc::collections::VecDeque;
use conquer_once::spin::Lazy;
use spinning_top::{Spinlock, SpinlockGuard};

static WOKEN_PIDS: Lazy<Spinlock<VecDeque<process::SlotId>>> =
    Lazy::new(|| Spinlock::new(VecDeque::new()));

pub(in crate::process) fn change_active_pid() {
    lock_queue().rotate_left(1);
}

pub(in crate::process) fn active_pid() -> process::SlotId {
    lock_queue()[0]
}

pub(in crate::process) fn pop() -> process::SlotId {
    lock_queue()
        .pop_front()
        .expect("All processes are terminated.")
}

pub(in crate::process) fn push(id: process::SlotId) {
    lock_queue().push_back(id);
}

fn lock_queue() -> SpinlockGuard<'static, VecDeque<process::SlotId>> {
    WOKEN_PIDS
        .try_lock()
        .expect("Failed to acquire the lock of `WOKEN_PIDS`.")
}
