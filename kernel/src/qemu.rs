// SPDX-License-Identifier: GPL-3.0-or-later

use qemu_exit::QEMUExit;

pub(crate) fn exit_success() -> ! {
    qemu_exit::X86::new(0xf4, 33).exit_success();
}

pub(crate) fn exit_failure() -> ! {
    qemu_exit::X86::new(0xf4, 33).exit_failure();
}
