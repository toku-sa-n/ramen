// SPDX-License-Identifier: GPL-3.0-or-later

use crate::mem::{allocator::page_box::PageBox, paging::pml4::PML4};
use x86_64::structures::paging::{PageTable, PageTableFlags};

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
