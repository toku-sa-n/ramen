// SPDX-License-Identifier: GPL-3.0-or-later

use super::{collections, collections::woken_pid, switch, Privilege, Process};
use alloc::collections::VecDeque;
use conquer_once::spin::Lazy;
use spinning_top::Spinlock;
pub use switch::switch;

static MESSAGE: Lazy<Spinlock<VecDeque<Message>>> = Lazy::new(|| Spinlock::new(VecDeque::new()));

pub fn main() -> ! {
    loop {
        while let Some(Message::Add(f, p)) = MESSAGE.lock().pop_front() {
            match p {
                Privilege::Kernel => push_process_to_queue(Process::kernel(f)),
                Privilege::User => push_process_to_queue(Process::user(f)),
            }
        }
    }
}

pub fn init() {
    push_process_to_queue(Process::user(main));
}

pub fn add(f: fn() -> !, p: Privilege) {
    MESSAGE.lock().push_back(Message::Add(f, p));
}

pub fn getpid() -> i32 {
    collections::process::handle_running(|p| p.id.as_i32())
}

fn push_process_to_queue(p: Process) {
    add_pid(p.id());
    add_process(p);
}

fn add_pid(id: super::Id) {
    woken_pid::add(id);
}

fn add_process(p: Process) {
    collections::process::add(p);
}

pub(super) fn loader(f: fn() -> !) -> ! {
    f()
}

enum Message {
    Add(fn() -> !, Privilege),
}
