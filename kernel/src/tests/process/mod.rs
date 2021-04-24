// SPDX-License-Identifier: GPL-3.0-or-later

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub(crate) static SWITCH_TEST_SUCCESS: AtomicBool = AtomicBool::new(false);

pub(crate) fn count_switch() {
    const EXIT_GOAL: usize = 500;
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed);

    if COUNTER.load(Ordering::Relaxed) >= EXIT_GOAL {
        SWITCH_TEST_SUCCESS.fetch_or(true, Ordering::Relaxed);
    }
}

pub(crate) fn exit_test() {
    syscalls::exit();
}
