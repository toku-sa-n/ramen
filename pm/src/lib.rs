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
    let mut processes = ProcessCollection::default();
    init(&mut processes);
    main_loop(&mut processes);
}

fn init(processes: &mut ProcessCollection) {
    add_initial_slots(processes);
}

fn add_initial_slots(processes: &mut ProcessCollection) {
    for i in 0..INITIAL_PROCESS_SLOT_NUMBER {
        processes.insert(i.try_into().unwrap(), Process::new(i.try_into().unwrap()));
    }
}

fn main_loop(processes: &mut ProcessCollection) {
    loop {
        loop_iteration(processes)
    }
}

fn loop_iteration(processes: &mut ProcessCollection) {
    let m = syscalls::receive_from_any();

    if let Some(syscalls::Ty::GetPid) = FromPrimitive::from_u64(m.body.0) {
        getpid(processes, m);
    } else {
        panic!("Unexpected message: {:?}", m);
    }
}

fn getpid(processes: &mut ProcessCollection, m: Message) {
    let pid = processes
        .get(m.header.sender)
        .expect("No such process.")
        .pid;
    let h = message::Header::new(PID);
    let b = message::Body(pid.try_into().unwrap(), 0, 0, 0, 0);

    let reply = Message::new(h, b);
    let to = m.header.sender;

    syscalls::send(reply, to);
}

#[derive(Default)]
struct ProcessCollection(BTreeMap<i32, Process>);
impl ProcessCollection {
    fn get(&self, k: i32) -> Option<&Process> {
        self.0.get(&k)
    }

    fn insert(&mut self, k: i32, p: Process) {
        self.0.insert(k, p);
    }
}

struct Process {
    pid: i32,
}
impl Process {
    fn new(pid: i32) -> Self {
        Self { pid }
    }
}
