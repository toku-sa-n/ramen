// SPDX-License-Identifier: GPL-3.0-or-later

mod manager;
mod stack_frame;

use crate::{
    mem::{allocator::page_box::PageBox, paging::pml4::PML4},
    tests,
    tss::TSS,
};
use common::constant::INTERRUPT_STACK;
use core::sync::atomic::{AtomicU64, Ordering};
use stack_frame::StackFrame;
use x86_64::{
    structures::paging::{PageTable, PageTableFlags},
    PhysAddr, VirtAddr,
};

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
        let pml4 = Pml4Creator::new().create();
        let pml4_addr = pml4.phys_addr();
        Process {
            id: Id::new(),
            stack: None,
            f,
            _pml4: pml4,
            pml4_addr,
            stack_frame: None,
        }
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

    fn as_u64(self) -> u64 {
        self.0
    }
}

struct Pml4Creator {
    pml4: PageBox<PageTable>,
}
impl Pml4Creator {
    fn new() -> Self {
        Self {
            pml4: PageBox::user(PageTable::new()),
        }
    }

    fn create(mut self) -> PageBox<PageTable> {
        self.enable_recursive_paging();
        self.map_kernel_area();
        self.pml4
    }

    fn enable_recursive_paging(&mut self) {
        let a = self.pml4.phys_addr();
        let f =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
        self.pml4[511].set_addr(a, f);
    }

    fn map_kernel_area(&mut self) {
        self.pml4[510] = PML4.lock().level_4_table()[510].clone();
    }
}
