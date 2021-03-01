// SPDX-License-Identifier: GPL-3.0-or-later

mod collections;
mod exit;
pub mod manager;
mod message;
mod page_table;
mod stack_frame;
mod switch;

use common::constant::INTERRUPT_STACK;
use core::{
    convert::TryInto,
    sync::atomic::{AtomicI32, Ordering},
};
use crossbeam_queue::ArrayQueue;
use message::Message;
use stack_frame::StackFrame;
use x86_64::{
    structures::paging::{PageSize, Size4KiB},
    PhysAddr, VirtAddr,
};

use crate::{mem::allocator::kpbox::KpBox, tss::TSS};

pub(super) fn init() {
    set_temporary_stack_frame();
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

    inbox: ArrayQueue<Message>,
}
impl Process {
    const STACK_SIZE: u64 = Size4KiB::SIZE * 12;
    const BOX_SIZE: usize = 128;

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

            inbox: ArrayQueue::new(Self::BOX_SIZE),
        }
    }

    fn id(&self) -> Id {
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
