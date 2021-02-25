// SPDX-License-Identifier: GPL-3.0-or-later

use crate::qemu;
use core::sync::atomic::Ordering;

mod mem;
pub mod process;

pub fn main() {
    self::mem::main();

    while !process::SWITCH_TEST_SUCCESS.load(Ordering::Relaxed) {}

    qemu::exit_success();
}
