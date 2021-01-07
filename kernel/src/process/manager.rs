// SPDX-License-Identifier: GPL-3.0-or-later

use super::{collections, collections::woken_pid, switch, Privilege, Process};
use alloc::collections::VecDeque;
use conquer_once::spin::Lazy;
use spinning_top::Spinlock;
pub use switch::switch;

pub(super) static MESSAGE: Lazy<Spinlock<VecDeque<Message>>> =
    Lazy::new(|| Spinlock::new(VecDeque::new()));

pub fn main() -> ! {
    loop {
        while let Some(Message::Add(f, p)) = MESSAGE.lock().pop_front() {
            match p {
                Privilege::Kernel => add(Process::kernel(f)),
                Privilege::User => add(Process::user(f)),
            }
        }
    }
}

pub fn init() {
    add(Process::user(main));
}

fn add(p: Process) {
    add_pid(p.id());
    add_process(p);
}

fn add_pid(id: super::Id) {
    woken_pid::add(id);
}

fn add_process(p: Process) {
    collections::process::add(p);
}

pub(crate) fn getpid() -> i32 {
    collections::process::handle_running(|p| p.id.as_i32())
}

pub(super) enum Message {
    Add(fn() -> !, Privilege),
    Exit,
}
