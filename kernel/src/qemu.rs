// SPDX-License-Identifier: GPL-3.0-or-later

use qemu_exit::QEMUExit;

pub fn exit() -> ! {
    qemu_exit::X86::new(0xf4, 33).exit_success();
}
