// SPDX-License-Identifier: GPL-3.0-or-later

mod collections;
mod context;
mod exit;
pub(crate) mod ipc;
mod page_table;
mod slot_id;
mod stack_frame;
pub(crate) mod switch;

use crate::tss::TSS;
use alloc::collections::VecDeque;
use common::constant::INTERRUPT_STACK;
use context::Context;
use core::convert::TryInto;
pub(crate) use slot_id::SlotId;
pub(crate) use switch::switch;
use x86_64::{
    structures::paging::{PageSize, Size4KiB},
    PhysAddr, VirtAddr,
};

pub(super) fn from_function(entry: fn(), name: &'static str) {
    let entry = VirtAddr::new((entry as usize).try_into().unwrap());

    push_process_to_queue(Process::new(entry, Privilege::Kernel, name));
}

pub(super) fn binary(name: &'static str, p: Privilege) {
    push_process_to_queue(Process::binary(name, p));
}

pub(crate) fn current_name() -> &'static str {
    collections::process::handle_running(|p| p.name)
}

fn get_slot_id() -> i32 {
    collections::woken_pid::active_pid()
}

fn block_running() {
    collections::woken_pid::pop();
}

fn push_process_to_queue(p: Process) {
    add_pid(p.id());
    add_process(p);
}

fn add_pid(id: SlotId) {
    collections::woken_pid::add(id);
}

fn add_process(p: Process) {
    collections::process::add(p);
}

fn set_temporary_stack_frame() {
    TSS.lock().interrupt_stack_table[0] = *INTERRUPT_STACK;
}

#[derive(Debug)]
pub(crate) struct Process {
    id: SlotId,

    context: Context,

    msg_ptr: Option<PhysAddr>,

    send_to: Option<SlotId>,
    receive_from: Option<ReceiveFrom>,

    pids_try_to_send_this_process: VecDeque<SlotId>,
    pids_try_to_receive_from_this_process: VecDeque<SlotId>,

    name: &'static str,
}
impl Process {
    const STACK_SIZE: u64 = Size4KiB::SIZE * 4;

    fn new(entry: VirtAddr, privilege: Privilege, name: &'static str) -> Self {
        todo!()
    }

    fn binary(name: &'static str, privilege: Privilege) -> Self {
        todo!()
    }

    fn id(&self) -> SlotId {
        self.id
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum Privilege {
    Kernel,
    User,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(crate) enum ReceiveFrom {
    Any,
    Id(SlotId),
}
