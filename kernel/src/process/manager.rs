// SPDX-License-Identifier: GPL-3.0-or-later

use super::{collections, collections::woken_pid, switch, Privilege, Process};

pub use super::exit::exit;
pub use switch::switch;

pub fn add(f: fn(), p: Privilege) {
    push_process_to_queue(Process::new(f, p));
}

pub fn getpid() -> i32 {
    collections::process::handle_running(|p| p.id.as_i32())
}

pub fn notify(pid: i32) {
    let _ = collections::process::handle(super::Id::from(pid), |p| {
        p.inbox.push(super::message::Message)
    });
}

pub fn notify_exists() -> bool {
    collections::process::handle_running(|p| p.inbox.pop()).is_some()
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
