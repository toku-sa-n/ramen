// SPDX-License-Identifier: GPL-3.0-or-later

use core::sync::atomic::Ordering;
use qemu_exit::QEMUExit;

mod mem;
pub mod process;
mod syscall;

pub fn main() {
    self::syscall::main();
    self::mem::main();

    while !process::SWITCH_TEST_SUCCESS.load(Ordering::Relaxed) {}

    qemu_exit::X86::new(0xf4, 33).exit_success();
}
