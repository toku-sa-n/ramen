// SPDX-License-Identifier: GPL-3.0-or-later

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub static SWITCH_TEST_SUCCESS: AtomicBool = AtomicBool::new(false);

pub fn count_switch() {
    const EXIT_GOAL: usize = 100;
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed);

    if COUNTER.load(Ordering::Relaxed) >= EXIT_GOAL {
        SWITCH_TEST_SUCCESS.fetch_or(true, Ordering::Relaxed);
    }
}

pub fn kernel_privilege_test() -> ! {
    loop {
        x86_64::instructions::hlt()
    }
}

pub fn exit_test() -> ! {
    syscalls::exit();
}
