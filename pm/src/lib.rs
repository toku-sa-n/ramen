// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]

const PID: i32 = 1;

pub fn main() {
    ensure_pid_is_correct();
}

fn ensure_pid_is_correct() {
    assert_eq!(syscalls::getpid(), PID, "Wrong PID");
}
