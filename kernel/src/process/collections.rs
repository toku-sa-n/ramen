// SPDX-License-Identifier: GPL-3.0-or-later

use super::Process;
use alloc::collections::{BTreeMap, VecDeque};
use conquer_once::spin::Lazy;
use spinning_top::Spinlock;

static PROCESSES: Spinlock<BTreeMap<super::Id, Process>> = Spinlock::new(BTreeMap::new());
static PIDS: Lazy<Spinlock<VecDeque<super::Id>>> = Lazy::new(|| Spinlock::new(VecDeque::new()));

fn add(p: Process) {
    let id = p.id();

    let mut is = PIDS
        .try_lock()
        .expect("Failed to acquire the lock of `PIDS`.");
    is.push_back(id);

    let mut ps = PROCESSES
        .try_lock()
        .expect("Failed to acquire the lock of `PROCESSES`.");
    ps.insert(id, p);
}
