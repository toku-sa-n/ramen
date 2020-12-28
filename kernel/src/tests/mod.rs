// SPDX-License-Identifier: GPL-3.0-or-later

use core::sync::atomic::Ordering;

pub mod process;

pub fn main() -> ! {
    while !process::SWITCH_TEST_SUCCESS.load(Ordering::Relaxed) {}

    use qemu_exit::QEMUExit;
    qemu_exit::X86::new(0xf4, 33).exit_success();
}
