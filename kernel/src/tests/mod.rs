// SPDX-License-Identifier: GPL-3.0-or-later

use crate::qemu;
use core::sync::atomic::Ordering;

pub mod process;
mod syscall;

pub fn main() {
    self::syscall::main();

    while !process::SWITCH_TEST_SUCCESS.load(Ordering::Relaxed) {}

    process::ipc::assert_test_completion();

    qemu::exit_success();
}
