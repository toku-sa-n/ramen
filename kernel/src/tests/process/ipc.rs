// SPDX-License-Identifier: GPL-3.0-or-later

// TODO: Use `fork` system call and communicate between parent and child.

use conquer_once::spin::OnceCell;
use core::sync::atomic::{AtomicI32, Ordering};
use message::Message;

static PROC1_PID: OnceCell<i32> = OnceCell::uninit();
static PROC2_PID: OnceCell<i32> = OnceCell::uninit();

static TEST_COMPLETED: AtomicI32 = AtomicI32::new(0);

pub(in crate::tests) fn assert_test_completion() {
    assert_eq!(
        TEST_COMPLETED.load(Ordering::Relaxed),
        2,
        "IPC test failed."
    );
}

pub fn proc_1() {
    PROC1_PID.init_once(syscalls::getpid);

    let mut m = Message::new(3, 1, 4, 1);

    while !PROC2_PID.is_initialized() {}
    let to = *PROC2_PID.get().expect("PROC2_PID is not initialized.");

    syscalls::send(m, to);

    syscalls::receive_from_any(&mut m);

    assert_eq!(m, Message::new(2, 0, 2, 1));

    TEST_COMPLETED.fetch_add(1, Ordering::Relaxed);
}

pub fn proc_2() {
    PROC2_PID.init_once(syscalls::getpid);

    let mut m = Message::default();
    syscalls::receive_from_any(&mut m);

    assert_eq!(m, Message::new(3, 1, 4, 1));

    let m = Message::new(2, 0, 2, 1);

    while !PROC1_PID.is_initialized() {}
    let to = *PROC1_PID.get().expect("PROC1_PID is not initialized.");

    syscalls::send(m, to);

    TEST_COMPLETED.fetch_add(1, Ordering::Relaxed);
}
