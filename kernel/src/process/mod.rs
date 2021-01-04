// SPDX-License-Identifier: GPL-3.0-or-later

mod manager;
mod stack_frame;

use crate::{
    mem::{allocator::page_box::PageBox, paging::pml4::PML4},
    tests,
    tss::TSS,
};
use common::constant::INTERRUPT_STACK;
use stack_frame::StackFrame;
use x86_64::{
    structures::paging::{PageTable, PageTableFlags},
    PhysAddr, VirtAddr,
};

pub fn init() {
    register_initial_interrupt_stack_table_addr();
}

pub fn add(p: Process) {
    manager::add_process(p);
}

pub fn switch() -> VirtAddr {
    if cfg!(feature = "qemu_test") {
        tests::process::count_switch();
    }
    manager::switch_process()
}

pub fn exit() -> ! {
    manager::exit();
}

fn register_initial_interrupt_stack_table_addr() {
    TSS.lock().interrupt_stack_table[0] = INTERRUPT_STACK;
}

pub struct Process {
    stack: Option<PageBox<[u8]>>,
    f: fn() -> !,
    _pml4: PageBox<PageTable>,
    pml4_addr: PhysAddr,
    stack_frame: Option<PageBox<StackFrame>>,
    running: bool,
}
impl Process {
    pub fn new(f: fn() -> !) -> Self {
        let pml4 = Pml4Creator::new().create();
        let pml4_addr = pml4.phys_addr();

        Process {
            stack: None,
            f,
            _pml4: pml4,
            pml4_addr,
            stack_frame: None,
            running: true,
        }
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

pub struct Pml4Creator {
    pml4: PageBox<PageTable>,
}
impl Pml4Creator {
    pub fn new() -> Self {
        Self {
            pml4: PageBox::user(PageTable::new()),
        }
    }

    pub fn create(mut self) -> PageBox<PageTable> {
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
