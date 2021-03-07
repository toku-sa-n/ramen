// SPDX-License-Identifier: GPL-3.0-or-later

mod collections;
mod exit;
pub(crate) mod ipc;
mod page_table;
mod stack_frame;
mod switch;

use crate::{mem::allocator::kpbox::KpBox, tss::TSS};
use alloc::collections::VecDeque;
use bitflags::bitflags;
use common::constant::INTERRUPT_STACK;
use core::{
    convert::TryInto,
    sync::atomic::{AtomicI32, Ordering},
};
pub use exit::exit;
use stack_frame::StackFrame;
pub use switch::switch;
use x86_64::{
    structures::paging::{PageSize, Size4KiB},
    PhysAddr, VirtAddr,
};

pub(crate) fn assign_rax_from_register() {
    let rax;
    unsafe { asm!("", out("rax") rax) }

    assign_rax(rax);
}

pub fn add(f: fn(), p: Privilege) {
    push_process_to_queue(Process::new(f, p));
}

pub fn getpid() -> i32 {
    collections::process::handle_running(|p| p.id.as_i32())
}

fn block_running() {
    collections::woken_pid::pop();
}

fn push_process_to_queue(p: Process) {
    add_pid(p.id());
    add_process(p);
}

fn add_pid(id: Id) {
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
pub struct Process {
    id: Id,
    f: fn(),
    tables: page_table::Collection,
    pml4_addr: PhysAddr,
    stack: KpBox<[u8]>,
    stack_frame: KpBox<StackFrame>,
    privilege: Privilege,

    flags: Flags,
    msg_ptr: Option<PhysAddr>,
    pids_try_to_send_this_process: VecDeque<i32>,
}
impl Process {
    const STACK_SIZE: u64 = Size4KiB::SIZE * 12;

    #[allow(clippy::too_many_lines)]
    fn new(f: fn(), privilege: Privilege) -> Self {
        let mut tables = page_table::Collection::default();
        let stack = KpBox::new_slice(0, Self::STACK_SIZE.try_into().unwrap());
        let stack_bottom = stack.virt_addr() + stack.bytes().as_usize();
        let stack_frame = KpBox::from(match privilege {
            Privilege::Kernel => StackFrame::kernel(f, stack_bottom),
            Privilege::User => StackFrame::user(f, stack_bottom),
        });
        tables.map_page_box(&stack);
        tables.map_page_box(&stack_frame);

        let pml4_addr = tables.pml4_addr();

        Process {
            id: Id::new(),
            f,
            tables,
            pml4_addr,
            stack,
            stack_frame,
            privilege,

            flags: Flags::empty(),
            msg_ptr: None,
            pids_try_to_send_this_process: VecDeque::new(),
        }
    }

    fn id(&self) -> Id {
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
pub enum Privilege {
    Kernel,
    User,
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
struct Id(i32);
impl Id {
    fn new() -> Self {
        static ID: AtomicI32 = AtomicI32::new(0);
        Self(ID.fetch_add(1, Ordering::Relaxed))
    }

    fn as_i32(self) -> i32 {
        self.0
    }
}
impl From<i32> for Id {
    fn from(id: i32) -> Self {
        Self(id)
    }
}
