// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;

use crate::mem::{allocator::page_box::PageBox, paging::pml4::PML4};
use alloc::vec::Vec;
use spinning_top::Spinlock;
use x86_64::{
    structures::paging::{
        page_table::PageTableEntry, PageSize, PageTable, PageTableFlags, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

static QUEUE: Spinlock<Vec<Process>> = Spinlock::new(Vec::new());

struct Process {
    pml4: PageBox<PageTable>,
    entry_addr: VirtAddr,
    rsp: VirtAddr,
    stack: PageBox<[u8]>,
}
impl Process {
    fn new(entry_addr: VirtAddr) -> Self {
        let stack = PageBox::new_slice(0, Size4KiB::SIZE.try_into().unwrap());
        Self {
            pml4: Pml4Creator::new().create(),
            entry_addr,
            rsp: stack.virt_addr() + stack.bytes().as_usize(),
            stack,
        }
    }
}

struct Pml4Creator {
    pml4: PageBox<PageTable>,
}
impl Pml4Creator {
    fn new() -> Self {
        Self {
            pml4: PageBox::new(PageTable::new()),
        }
    }

    fn create(mut self) -> PageBox<PageTable> {
        self.enable_recursive_mapping();
        self.map_kernel_regions();
        self.pml4
    }

    fn enable_recursive_mapping(&mut self) {
        let a = self.pml4.phys_addr();
        self.pml4[511].set_addr(a, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
    }

    fn map_kernel_regions(&mut self) {
        // Kernel region starts from `0xffff_ffff_8000_0000`.
        let p3 = PML4.lock().level_4_table()[510].addr();
        self.pml4[510].set_addr(p3, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
    }
}
