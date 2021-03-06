// SPDX-License-Identifier: GPL-3.0-or-later

use crate::qemu;
use core::sync::atomic::Ordering;

pub(crate) mod process;
mod syscall;

pub(crate) fn main() {
    self::syscall::main();

    while !process::SWITCH_TEST_SUCCESS.load(Ordering::Relaxed) {}

    qemu::exit_success();
}
