// SPDX-License-Identifier: GPL-3.0-or-later

extern "C" {
    fn exit_qemu_as_success() -> !;
    fn exit_qemu_as_failure() -> !;
}

pub(crate) fn exit_success() -> ! {
    unsafe { exit_qemu_as_success() }
}

pub(crate) fn exit_failure() -> ! {
    unsafe { exit_qemu_as_failure() }
}
