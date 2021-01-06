// SPDX-License-Identifier: GPL-3.0-or-later

pub mod manager;
mod stack_frame;

use crate::{
    mem::{allocator::page_box::PageBox, paging::pml4::PML4},
    tests,
    tss::TSS,
};
use common::constant::INTERRUPT_STACK;
use core::sync::atomic::{AtomicI32, Ordering};
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

pub fn getpid() -> i32 {
    manager::getpid()
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
    privilege: Privilege,
}
impl Process {
    pub fn kernel(f: fn() -> !) -> Self {
        Self::new(f, Privilege::Kernel)
    }

    pub fn user(f: fn() -> !) -> Self {
        Self::new(f, Privilege::User)
    }

    fn new(f: fn() -> !, privilege: Privilege) -> Self {
        let pml4 = Pml4Creator::new().create();
        let pml4_addr = pml4.phys_addr();
        Process {
            id: Id::new(),
            stack: None,
            f,
            _pml4: pml4,
            pml4_addr,
            stack_frame: None,
            privilege,
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

enum Privilege {
    Kernel,
    User,
}
