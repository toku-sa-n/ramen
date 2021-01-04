// SPDX-License-Identifier: GPL-3.0-or-later

use crate::process::Process;
use alloc::collections::BTreeMap;
use spinning_top::Spinlock;

static PROCESSES: Spinlock<BTreeMap<crate::process::Id, Process>> = Spinlock::new(BTreeMap::new());

fn add(p: Process) {
    let id = p.id();
    let mut ps = PROCESSES
        .try_lock()
        .expect("Failed to acquire the lock of `PROCESSES`.");

    ps.insert(id, p);
}
