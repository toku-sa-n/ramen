// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![feature(const_btree_new)]
// A workaround for the `derive_builder` crate.
#![allow(clippy::default_trait_access)]

extern crate alloc;

use futures_intrusive::sync::{GenericMutex, GenericMutexGuard};
use multitask::{executor::Executor, task::Task};
use spinning_top::RawSpinlock;

pub(crate) type Futurelock<T> = GenericMutex<RawSpinlock, T>;
pub(crate) type FuturelockGuard<'a, T> = GenericMutexGuard<'a, RawSpinlock, T>;

mod device;
mod multitask;

#[no_mangle]
pub fn main() {
    ralib::init();

    multitask::add(Task::new(crate::device::pci::xhci::task()));

    let mut executor = Executor::new();
    executor.run();
}
