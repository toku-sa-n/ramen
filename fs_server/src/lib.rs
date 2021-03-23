// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]

const PID: i32 = 1;

pub fn main() {
    ensure_pid_is_correct();
}

fn ensure_pid_is_correct() {
    let i = syscalls::getpid();
    assert_eq!(i, PID, "Wrong PID for the file system server: {}", i);
}
