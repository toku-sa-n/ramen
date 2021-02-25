// SPDX-License-Identifier: GPL-3.0-or-later

use conquer_once::spin::Lazy;
use spinning_top::Spinlock;
use x86_64::{
    structures::paging::{
        page_table::PageTableEntry, Mapper, Page, PageTableFlags, PhysFrame, RecursivePageTable,
        Translate,
    },
    PhysAddr, VirtAddr,
};

use crate::allocator::phys::FRAME_MANAGER;

const RECURSIVE_ENTRY: VirtAddr = VirtAddr::new_truncate(0xffff_ffff_ffff_f000);

pub(crate) static PML4: Lazy<Spinlock<RecursivePageTable>> = Lazy::new(|| unsafe {
    Spinlock::new(
        (RecursivePageTable::new(&mut *(RECURSIVE_ENTRY.as_mut_ptr())))
            .expect("PML4 has no recursive entry."),
    )
});

pub fn mark_pages_as_unused() {
    let mut p4 = PML4.lock();
    let p = p4.level_4_table();

    // Entry 510 and 511 are used by kernel.
    for i in 0..510 {
        p[i].set_unused();
    }
}

pub fn translate(v: VirtAddr) -> Option<PhysAddr> {
    PML4.lock().translate_addr(v)
}

pub fn entries_of_kernel_mapping() -> (PageTableEntry, PageTableEntry) {
    let mut p = PML4.lock();
    let p = p.level_4_table();
    (p[510].clone(), p[511].clone())
}

pub(crate) unsafe fn map_to(p: Page, pf: PhysFrame, f: PageTableFlags) {
    PML4.lock()
        .map_to(p, pf, f, &mut *FRAME_MANAGER.lock())
        .expect("Failed to map a page.")
        .flush()
}

pub(crate) fn unmap(p: Page) {
    let (_, f) = PML4.lock().unmap(p).expect("Failed to unmap a page.");
    f.flush();
}
