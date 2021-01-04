// SPDX-License-Identifier: GPL-3.0-or-later

use crate::process;
use alloc::collections::VecDeque;
use conquer_once::spin::Lazy;
use spinning_top::Spinlock;

static PIDS: Lazy<Spinlock<VecDeque<process::Id>>> = Lazy::new(|| Spinlock::new(VecDeque::new()));

fn add(id: process::Id) {
    let mut ps = PIDS
        .try_lock()
        .expect("Failed to acquire the lock of `PIDS`.");
    ps.push_back(id);
}
