// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]

extern crate alloc;

use alloc::collections::BTreeSet;
use log::info;

#[no_mangle]
pub fn main() {
    ralib::init();

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
        c.insert(k);
        k += 1;
    }

    info!("FS: Synced.");
    info!("FS: Number of processes: {}", k);
}

#[derive(Default)]
struct ProcessCollection(BTreeSet<i32>);
impl ProcessCollection {
    fn insert(&mut self, k: i32) {
        self.0.insert(k);
    }
}
