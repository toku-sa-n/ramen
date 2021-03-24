// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use core::convert::TryInto;
use message::Message;
use num_traits::FromPrimitive;

const PID: i32 = 1;
const INITIAL_PROCESS_SLOT_NUMBER: usize = 200;

pub fn main() {
    let mut processes = BTreeMap::new();
    init(&mut processes);
    main_loop(&mut processes);
}

fn init(processes: &mut BTreeMap<i32, Process>) {
    add_initial_slots(processes);
}

fn add_initial_slots(processes: &mut BTreeMap<i32, Process>) {
    for i in 0..INITIAL_PROCESS_SLOT_NUMBER {
        processes.insert(i.try_into().unwrap(), Process::new(i.try_into().unwrap()));
    }
}

fn main_loop(processes: &mut BTreeMap<i32, Process>) {
    loop {
        loop_iteration(processes)
    }
}

fn loop_iteration(processes: &mut BTreeMap<i32, Process>) {
    let m = syscalls::receive_from_any();

    if let Some(syscalls::Ty::GetPid) = FromPrimitive::from_u64(m.body.0) {
        getpid(processes, m);
    }
}

fn getpid(processes: &mut BTreeMap<i32, Process>, m: Message) {
    let pid = processes
        .get(&m.header.sender)
        .expect("No such process.")
        .pid;
    let h = message::Header::new(PID);
    let b = message::Body(pid.try_into().unwrap(), 0, 0, 0, 0);

    let reply = Message::new(h, b);
    let to = m.header.sender;

    syscalls::send(reply, to);
}

struct Process {
    pid: i32,
}
impl Process {
    fn new(pid: i32) -> Self {
        Self { pid }
    }
}
