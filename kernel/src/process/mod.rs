// SPDX-License-Identifier: GPL-3.0-or-later

mod collections;
mod exit;
pub(crate) mod ipc;
mod page_table;
mod slot_id;
mod stack_frame;
pub(crate) mod switch;

use crate::{mem::allocator::kpbox::KpBox, tss::TSS};
use alloc::collections::VecDeque;
use bitflags::bitflags;
use common::constant::INTERRUPT_STACK;
use core::convert::TryInto;
pub(crate) use exit::exit;
pub(crate) use slot_id::SlotId;
use stack_frame::StackFrame;
pub(crate) use switch::switch;
use x86_64::{
    structures::paging::{PageSize, Size4KiB},
    PhysAddr, VirtAddr,
};

pub(crate) fn assign_rax_from_register() {
    let rax;
    unsafe { asm!("", out("rax") rax) }

    assign_rax(rax);
}

pub(super) fn add(entry: fn(), p: Privilege, name: &'static str) {
    let entry = VirtAddr::new((entry as usize).try_into().unwrap());
    push_process_to_queue(Process::new(entry, p, name));
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

pub(super) fn loader(f: fn()) -> ! {
    f();
    syscalls::exit();
}

fn assign_rax(rax: u64) {
    collections::process::handle_running_mut(|p| (*p.stack_frame).regs.rax = rax);
}

fn set_temporary_stack_frame() {
    TSS.lock().interrupt_stack_table[0] = INTERRUPT_STACK;
}

#[derive(Debug)]
pub(crate) struct Process {
    id: SlotId,
    entry: VirtAddr,
    tables: page_table::Collection,
    pml4_addr: PhysAddr,
    stack: KpBox<[u8]>,
    stack_frame: KpBox<StackFrame>,
    privilege: Privilege,
    binary: Option<KpBox<[u8]>>,

    flags: Flags,
    msg_ptr: Option<PhysAddr>,
    pids_try_to_send_this_process: VecDeque<SlotId>,

    name: &'static str,
}
impl Process {
    const STACK_SIZE: u64 = Size4KiB::SIZE * 8;

    #[allow(clippy::too_many_lines)]
    fn new(entry: VirtAddr, privilege: Privilege, name: &'static str) -> Self {
        let mut tables = page_table::Collection::default();
        let stack = KpBox::new_slice(0, Self::STACK_SIZE.try_into().unwrap());
        let stack_bottom = stack.virt_addr() + stack.bytes().as_usize();
        let stack_frame = KpBox::from(match privilege {
            Privilege::Kernel => StackFrame::kernel(entry, stack_bottom),
            Privilege::User => StackFrame::user(entry, stack_bottom),
        });
        tables.map_page_box(&stack);
        tables.map_page_box(&stack_frame);

        let pml4_addr = tables.pml4_addr();

        Process {
            id: slot_id::generate(),
            entry,
            tables,
            pml4_addr,
            stack,
            stack_frame,
            privilege,
            binary: None,

            flags: Flags::empty(),
            msg_ptr: None,
            pids_try_to_send_this_process: VecDeque::new(),
            name,
        }
    }

    #[allow(clippy::too_many_lines)]
    fn binary(name: &'static str, privilege: Privilege) -> Self {
        let mut tables = page_table::Collection::default();
        let (content, entry) = tables.map_elf(name);

        let stack = KpBox::new_slice(0, Self::STACK_SIZE.try_into().unwrap());
        let stack_bottom = stack.virt_addr() + stack.bytes().as_usize();
        let stack_frame = KpBox::from(match privilege {
            Privilege::Kernel => StackFrame::kernel(entry, stack_bottom),
            Privilege::User => StackFrame::user(entry, stack_bottom),
        });
        tables.map_page_box(&stack);
        tables.map_page_box(&stack_frame);

        let pml4_addr = tables.pml4_addr();

        Self {
            id: slot_id::generate(),
            entry,
            tables,
            pml4_addr,
            stack,
            stack_frame,
            privilege,
            binary: Some(content),

            flags: Flags::empty(),
            msg_ptr: None,
            pids_try_to_send_this_process: VecDeque::new(),
            name,
        }
    }

    fn id(&self) -> SlotId {
        self.id
    }

    fn waiting_message(&self) -> bool {
        self.flags.contains(Flags::RECEIVING)
    }

    fn stack_frame_top_addr(&self) -> VirtAddr {
        self.stack_frame.virt_addr()
    }

    fn stack_frame_bottom_addr(&self) -> VirtAddr {
        let b = self.stack_frame.bytes();
        self.stack_frame_top_addr() + b.as_usize()
    }
}

bitflags! {
    struct Flags:u32{
        const SENDING=0b0001;
        const RECEIVING=0b0010;
    }
}

#[derive(Debug)]
pub(crate) enum Privilege {
    Kernel,
    User,
}
