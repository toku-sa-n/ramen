// SPDX-License-Identifier: GPL-3.0-or-later

use crate::mem::{allocator::kpbox::KpBox, paging::pml4::PML4};
use alloc::collections::BTreeMap;
use core::convert::TryFrom;
use x86_64::{
    structures::paging::{
        Page, PageSize, PageTable, PageTableFlags, PageTableIndex, PhysFrame, Size4KiB,
    },
    PhysAddr,
};

#[derive(Debug)]
pub(super) struct Collection {
    pml4: KpBox<PageTable>,
    pdpt: BTreeMap<PageTableIndex, KpBox<PageTable>>,
    pd: BTreeMap<PageTableIndex, KpBox<PageTable>>,
    pt: BTreeMap<PageTableIndex, KpBox<PageTable>>,
}
impl Collection {
    pub(super) fn pml4_addr(&self) -> PhysAddr {
        self.pml4.phys_addr()
    }

    pub(super) fn map_page_box<T: ?Sized>(&mut self, b: &KpBox<T>) {
        for i in 0..b.bytes().as_num_of_pages::<Size4KiB>().as_usize() {
            let off = Size4KiB::SIZE * u64::try_from(i).unwrap();
            let v = Page::from_start_address(b.virt_addr() + off).expect("Page is not aligned.");
            let p =
                PhysFrame::from_start_address(b.phys_addr() + off).expect("Frame is not aligned.");
            self.map(v, p);
        }
    }

    fn map(&mut self, v: Page<Size4KiB>, p: PhysFrame) {
        let Self { pml4, pdpt, pd, pt } = self;

        let pml4_i = v.p4_index();
        let pdpt_i = v.p3_index();
        let dir_i = v.p2_index();
        let table_i = v.p1_index();

        let p3 = pdpt
            .entry(pml4_i)
            .or_insert_with(|| Self::create(pml4, pml4_i));

        let p2 = pd.entry(pdpt_i).or_insert_with(|| Self::create(p3, pdpt_i));

        let p1 = pt.entry(dir_i).or_insert_with(|| Self::create(p2, dir_i));

        p1[table_i].set_addr(p.start_address(), Self::flags());
    }

    fn create(parent: &mut PageTable, i: PageTableIndex) -> KpBox<PageTable> {
        let t = KpBox::from(PageTable::new());
        Self::map_transition(parent, &t, i);
        t
    }

    fn map_transition(from: &mut PageTable, to: &KpBox<PageTable>, i: PageTableIndex) {
        from[i].set_addr(to.phys_addr(), Self::flags());
    }

    fn flags() -> PageTableFlags {
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE
    }
}
impl Default for Collection {
    fn default() -> Self {
        Self {
            pml4: Pml4Creator::default().create(),
            pdpt: BTreeMap::default(),
            pd: BTreeMap::default(),
            pt: BTreeMap::default(),
        }
    }
}

#[derive(Default)]
struct Pml4Creator {
    pml4: KpBox<PageTable>,
}
impl Pml4Creator {
    fn create(mut self) -> KpBox<PageTable> {
        self.enable_recursive_paging();
        self.map_kernel_area();
        self.pml4
    }

    fn enable_recursive_paging(&mut self) {
        let a = self.pml4.phys_addr();
        let f =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
        self.pml4[510].set_addr(a, f);
    }

    fn map_kernel_area(&mut self) {
        self.pml4[511] = PML4.lock().level_4_table()[511].clone();
    }
}
