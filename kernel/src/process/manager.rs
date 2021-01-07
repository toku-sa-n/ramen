// SPDX-License-Identifier: GPL-3.0-or-later

use super::{collections, collections::woken_pid, switch, Privilege, Process};
use crate::tss::TSS;
use alloc::collections::VecDeque;
use common::constant::INTERRUPT_STACK;
use conquer_once::spin::Lazy;
use spinning_top::Spinlock;

pub use super::exit::exit;
pub use switch::switch;

static MESSAGE: Lazy<Spinlock<VecDeque<Message>>> = Lazy::new(|| Spinlock::new(VecDeque::new()));

pub fn main() {
    loop {
        while let Some(m) = MESSAGE.lock().pop_front() {
            match m {
                Message::Add(f, p) => match p {
                    Privilege::Kernel => push_process_to_queue(Process::kernel(f)),
                    Privilege::User => push_process_to_queue(Process::user(f)),
                },
                Message::Exit(id) => collections::process::remove(id),
            }
        }
    }
}

pub fn init() {
    set_temporary_stack_frame();
    push_process_to_queue(Process::user(main));
}

pub fn add(f: fn(), p: Privilege) {
    send_message(Message::Add(f, p));
}

pub fn getpid() -> i32 {
    collections::process::handle_running(|p| p.id.as_i32())
}

pub(super) fn send_message(m: Message) {
    MESSAGE.lock().push_back(m);
}

pub(super) fn set_temporary_stack_frame() {
    TSS.lock().interrupt_stack_table[0] = INTERRUPT_STACK;
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

pub(super) fn loader(f: fn()) -> ! {
    f();
    syscalls::exit();
}

pub(super) enum Message {
    Add(fn(), Privilege),
    Exit(super::Id),
}
