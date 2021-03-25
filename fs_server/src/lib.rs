// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]

#[macro_use]
extern crate log;
extern crate alloc;

use alloc::collections::BTreeMap;

pub fn main() {
    let mut c = ProcessCollection::default();
    init(&mut c);
}

fn init(c: &mut ProcessCollection) {
    let mut m;
    let mut k = 0;
    while {
        m = syscalls::receive_from_any();
        m.body.1 > 0
    } {
        c.insert(k, Process);
        k += 1;
    }

    info!("FS: Synced.");
    info!("FS: Number of processes: {}", k);
}

#[derive(Default)]
struct ProcessCollection(BTreeMap<i32, Process>);
impl ProcessCollection {
    fn insert(&mut self, k: i32, p: Process) {
        self.0.insert(k, p);
    }
}

struct Process;
