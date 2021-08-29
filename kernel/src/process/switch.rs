// SPDX-License-Identifier: GPL-3.0-or-later

use super::collections::woken_pid;
use crate::tests;

pub(crate) fn switch() {
    if cfg!(feature = "qemu_test") {
        tests::process::count_switch();
    }

    change_current_process();
}

fn change_current_process() {
    woken_pid::change_active_pid();
}
