// SPDX-License-Identifier: GPL-3.0-or-later

mod collections;
mod exit;
pub(crate) mod ipc;
mod page_table;
mod slot_id;
mod stack_frame;
pub(crate) mod switch;

use {
    crate::{mem::allocator::kpbox::KpBox, tss::TSS},
    alloc::collections::VecDeque,
    core::convert::TryInto,
    predefined_mmap::INTERRUPT_STACK,
    stack_frame::StackFrame,
    x86_64::{
        structures::paging::{PageSize, PhysFrame, Size4KiB},
        PhysAddr, VirtAddr,
    },
};
pub(crate) use {exit::exit_process, slot_id::SlotId, switch::switch};

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
    collections::woken_pid::push(id);
}

fn add_process(p: Process) {
    collections::process::add(p);
}

pub(super) fn loader(f: fn()) -> ! {
    f();
    syscalls::exit();
}

pub(crate) fn assign_to_rax(rax: u64) {
    collections::process::handle_running_mut(|p| (*p.stack_frame).regs.rax = rax);
}

fn set_temporary_stack_frame() {
    TSS.lock().interrupt_stack_table[0] = *INTERRUPT_STACK;
}

#[derive(Debug)]
pub(crate) struct Process {
    id: SlotId,
    _tables: page_table::Collection,
    pml4: PhysFrame,
    _stack: KpBox<[u8]>,
    stack_frame: KpBox<StackFrame>,
    _binary: Option<KpBox<[u8]>>,

    msg_ptr: Option<PhysAddr>,

    send_to: Option<SlotId>,
    receive_from: Option<ReceiveFrom>,

    pids_try_to_send_this_process: VecDeque<SlotId>,
    _pids_try_to_receive_from_this_process: VecDeque<SlotId>,

    name: &'static str,
}
impl Process {
    const STACK_SIZE: u64 = Size4KiB::SIZE * 4;

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

        let pml4 = tables.pml4_frame();

        Process {
            id: slot_id::generate(),
            _tables: tables,
            pml4,
            _stack: stack,
            stack_frame,
            _binary: None,

            msg_ptr: None,

            send_to: None,
            receive_from: None,

            pids_try_to_send_this_process: VecDeque::new(),
            _pids_try_to_receive_from_this_process: VecDeque::new(),
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

        let pml4 = tables.pml4_frame();

        Self {
            id: slot_id::generate(),
            _tables: tables,
            pml4,
            _stack: stack,
            stack_frame,
            _binary: Some(content),

            msg_ptr: None,

            send_to: None,
            receive_from: None,

            pids_try_to_send_this_process: VecDeque::new(),
            _pids_try_to_receive_from_this_process: VecDeque::new(),
            name,
        }
    }

    fn id(&self) -> SlotId {
        self.id
    }

    fn stack_frame_top_addr(&self) -> VirtAddr {
        self.stack_frame.virt_addr()
    }

    fn stack_frame_bottom_addr(&self) -> VirtAddr {
        let b = self.stack_frame.bytes();
        self.stack_frame_top_addr() + b.as_usize()
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
