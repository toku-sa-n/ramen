// SPDX-License-Identifier: GPL-3.0-or-later

mod creator;
mod manager;
mod stack_frame;

use core::sync::atomic::{AtomicU64, Ordering};

use crate::{mem::allocator::page_box::PageBox, tests, tss::TSS};
use common::constant::INTERRUPT_STACK;
use creator::Creator;
use stack_frame::StackFrame;
use x86_64::{structures::paging::PageTable, PhysAddr, VirtAddr};

pub fn init() {
    register_initial_interrupt_stack_table_addr();
}

pub fn add(p: Process) {
    manager::add(p);
}

pub fn switch() -> VirtAddr {
    if cfg!(feature = "qemu_test") {
        tests::process::count_switch();
    }
    manager::switch()
}

fn register_initial_interrupt_stack_table_addr() {
    TSS.lock().interrupt_stack_table[0] = INTERRUPT_STACK;
}

pub struct Process {
    id: Id,
    stack: Option<PageBox<[u8]>>,
    f: fn() -> !,
    _pml4: PageBox<PageTable>,
    pml4_addr: PhysAddr,
    stack_frame: Option<PageBox<StackFrame>>,
}
impl Process {
    pub fn new(f: fn() -> !) -> Self {
        Creator::new(f).create()
    }

    fn id(&self) -> Id {
        self.id
    }

    fn stack_frame_top_addr(&self) -> VirtAddr {
        self.stack_frame().virt_addr()
    }

    fn stack_frame_bottom_addr(&self) -> VirtAddr {
        let b = self.stack_frame().bytes();
        self.stack_frame_top_addr() + b.as_usize()
    }

    fn stack_frame(&self) -> &PageBox<StackFrame> {
        self.stack_frame
            .as_ref()
            .expect("Stack frame is not created")
    }
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq)]
struct Id(u64);
impl Id {
    fn new() -> Self {
        static ID: AtomicU64 = AtomicU64::new(0);
        Self(ID.fetch_add(1, Ordering::Relaxed))
    }

    fn as_u64(&self) -> u64 {
        self.0
    }
}
