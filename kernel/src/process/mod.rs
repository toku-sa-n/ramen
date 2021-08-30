// SPDX-License-Identifier: GPL-3.0-or-later

mod collections;
mod context;
mod exit;
pub(crate) mod ipc;
mod page_table;
mod slot_id;
pub(crate) mod switch;

use crate::{mem::allocator::kpbox::KpBox, tss::TSS};
use alloc::collections::VecDeque;
use common::constant::INTERRUPT_STACK;
use context::Context;
use core::convert::TryInto;
pub(crate) use exit::exit_process;
pub(crate) use slot_id::SlotId;
pub(crate) use switch::switch;
use x86_64::{
    registers::control::Cr3,
    structures::paging::{PageSize, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};

pub(super) fn from_function(entry: fn(), name: &'static str) {
    let entry = VirtAddr::new((entry as usize).try_into().unwrap());
    push_process_to_queue(Process::new(entry, Privilege::Kernel, name));
}

pub(super) fn binary(name: &'static str, p: Privilege) {
    push_process_to_queue(Process::binary(name, p));
}

pub(super) fn idle() {
    add_process(Process::idle());
}

pub(crate) fn current_name() -> &'static str {
    collections::process::handle_running(|p| p.name)
}

fn get_slot_id() -> i32 {
    collections::woken_pid::active_pid()
}

fn sleep_current() {
    log::info!("Sleep!!!");
    collections::process::handle_running_mut(|p| p.state = State::Suspend);
    switch();
}

fn wake(id: SlotId) {
    log::info!("Waking {}", id);
    collections::process::handle_mut(id, |p| p.state = State::Ready);
    collections::woken_pid::add(id);
    switch();
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

fn set_temporary_stack_frame() {
    TSS.lock().interrupt_stack_table[0] = *INTERRUPT_STACK;
}

#[derive(Debug)]
pub(crate) struct Process {
    id: SlotId,
    entry: VirtAddr,
    tables: page_table::Collection,
    pml4: PhysFrame,
    stack: KpBox<[u8]>,
    privilege: Privilege,
    binary: Option<KpBox<[u8]>>,

    context: Context,

    kernel_stack: [u8; 4096],

    msg_ptr: Option<PhysAddr>,

    send_to: Option<SlotId>,
    receive_from: Option<ReceiveFrom>,

    pids_try_to_send_this_process: VecDeque<SlotId>,
    pids_try_to_receive_from_this_process: VecDeque<SlotId>,

    name: &'static str,

    state: State,
}
impl Process {
    const STACK_SIZE: u64 = Size4KiB::SIZE * 4;

    #[allow(clippy::too_many_lines)]
    fn new(entry: VirtAddr, privilege: Privilege, name: &'static str) -> Self {
        let mut tables = page_table::Collection::default();
        let stack = KpBox::new_slice(0, Self::STACK_SIZE.try_into().unwrap());

        tables.map_page_box(&stack);

        let context = match privilege {
            Privilege::Kernel => Context::kernel(
                entry,
                tables.pml4_frame(),
                stack.virt_addr() + Self::STACK_SIZE,
            ),
            Privilege::User => Context::user(
                entry,
                tables.pml4_frame(),
                stack.virt_addr() + Self::STACK_SIZE,
            ),
        };

        let pml4 = tables.pml4_frame();

        Process {
            id: slot_id::generate(),
            entry,
            tables,
            pml4,
            stack,
            privilege,
            binary: None,

            context,

            kernel_stack: [0; 4096],

            msg_ptr: None,

            send_to: None,
            receive_from: None,

            pids_try_to_send_this_process: VecDeque::new(),
            pids_try_to_receive_from_this_process: VecDeque::new(),
            name,

            state: State::Ready,
        }
    }

    #[allow(clippy::too_many_lines)]
    fn binary(name: &'static str, privilege: Privilege) -> Self {
        let mut tables = page_table::Collection::default();
        let (binary, entry) = tables.map_elf(name);

        let stack = KpBox::new_slice(0, Self::STACK_SIZE.try_into().unwrap());

        tables.map_page_box(&stack);

        log::info!("{:?}", tables.pml4_frame());

        let context = match privilege {
            Privilege::Kernel => Context::kernel(
                entry,
                tables.pml4_frame(),
                stack.virt_addr() + Self::STACK_SIZE,
            ),
            Privilege::User => Context::user(
                entry,
                tables.pml4_frame(),
                stack.virt_addr() + Self::STACK_SIZE,
            ),
        };

        let pml4 = tables.pml4_frame();

        Self {
            id: slot_id::generate(),
            entry,
            tables,
            pml4,
            stack,
            privilege,
            binary: Some(binary),

            context,

            kernel_stack: [0; 4096],

            msg_ptr: None,

            send_to: None,
            receive_from: None,

            pids_try_to_send_this_process: VecDeque::new(),
            pids_try_to_receive_from_this_process: VecDeque::new(),
            name,

            state: State::Ready,
        }
    }

    #[allow(clippy::too_many_lines)]
    fn idle() -> Self {
        static STACK: [u8; 64] = [0; 64];

        let (pml4, _) = Cr3::read();

        let context = Context::kernel(VirtAddr::zero(), pml4, VirtAddr::from_ptr(&STACK) + 64_u64);

        Self {
            id: 0,
            entry: VirtAddr::zero(),
            tables: page_table::Collection::default(),
            pml4,
            stack: KpBox::new_slice(0, 1),
            privilege: Privilege::Kernel,
            binary: None,
            context,
            kernel_stack: [0; 4096],
            msg_ptr: None,
            send_to: None,
            receive_from: None,
            pids_try_to_send_this_process: VecDeque::new(),
            pids_try_to_receive_from_this_process: VecDeque::new(),
            name: "Idle",

            state: State::Current,
        }
    }

    fn id(&self) -> SlotId {
        self.id
    }

    fn kernel_stack_bottom(&self) -> VirtAddr {
        VirtAddr::from_ptr(&self.kernel_stack) + 4096_u64
    }
}

#[derive(Debug)]
pub(crate) enum Privilege {
    Kernel,
    User,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(crate) enum ReceiveFrom {
    Any,
    Id(SlotId),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum State {
    Ready,
    Suspend,
    Current,
}
