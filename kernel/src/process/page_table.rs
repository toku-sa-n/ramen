// SPDX-License-Identifier: GPL-3.0-or-later

use crate::mem::{allocator::kpbox::KpBox, paging::pml4::PML4};
use alloc::collections::BTreeMap;
use core::convert::{TryFrom, TryInto};
use os_units::{Bytes, NumOfPages};
use x86_64::{
    structures::paging::{
        Page, PageSize, PageTable, PageTableFlags, PageTableIndex, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};
use xmas_elf::{program::ProgramHeader, ElfFile};

#[derive(Debug)]
pub(super) struct Collection {
    pml4: KpBox<PageTable>,
    pdpt_collection: BTreeMap<PhysAddr, KpBox<PageTable>>,
    pd_collection: BTreeMap<PhysAddr, KpBox<PageTable>>,
    pt_collection: BTreeMap<PhysAddr, KpBox<PageTable>>,
}
impl Collection {
    pub(super) fn pml4_addr(&self) -> PhysAddr {
        self.pml4.phys_addr()
    }

    pub(super) fn map_page_box(&mut self, b: &KpBox<impl ?Sized>) {
        for i in 0..b.bytes().as_num_of_pages::<Size4KiB>().as_usize() {
            let off = Size4KiB::SIZE * u64::try_from(i).unwrap();
            let v = Page::from_start_address(b.virt_addr() + off).expect("Page is not aligned.");
            let p =
                PhysFrame::from_start_address(b.phys_addr() + off).expect("Frame is not aligned.");
            self.map(v, p);
        }
    }

    pub(super) fn map_elf(&mut self, raw: &KpBox<[u8]>) {
        let elf_file = ElfFile::new(raw).expect("Not a ELF file.");
        for p in elf_file.program_iter() {
            self.map_program_header(p, raw);
        }
    }

    fn map_program_header(&mut self, ph: ProgramHeader, raw: &KpBox<[u8]>) {
        let virt_bottom = Self::segment_page_aligned_start_addr(ph).as_u64();
        let virt_top = Self::segment_page_aligned_end_addr(ph).as_u64();

        let num_of_pages =
            Bytes::new((virt_top - virt_bottom).try_into().unwrap()).as_num_of_pages::<Size4KiB>();

        for i in 0..num_of_pages.as_usize() {
            let offset = NumOfPages::<Size4KiB>::new(i).as_bytes().as_usize();
            let v = VirtAddr::new(ph.virtual_addr()) + offset;
            let v = Page::from_start_address(v).expect("This address is not aligned.");

            let p = raw.phys_addr() + ph.offset() + offset;
            let p = PhysFrame::from_start_address(p).expect("This address is not aligned.");

            self.map(v, p);
        }
    }

    fn segment_page_aligned_start_addr(ph: ProgramHeader) -> VirtAddr {
        let a = VirtAddr::new(ph.virtual_addr());
        assert!(
            a.is_aligned(Size4KiB::SIZE),
            "The start address of a segment is not page-aligned."
        );
        a
    }

    fn segment_page_aligned_end_addr(ph: ProgramHeader) -> VirtAddr {
        let a = Self::segment_page_aligned_start_addr(ph) + ph.mem_size();
        a.align_up(Size4KiB::SIZE)
    }

    fn map(&mut self, v: Page<Size4KiB>, p: PhysFrame) {
        let Self {
            pml4,
            pdpt_collection,
            pd_collection,
            pt_collection,
        } = self;

        let [pml4_i, pdpt_i, dir_i, table_i] =
            [v.p4_index(), v.p3_index(), v.p2_index(), v.p1_index()];
        let indexes = [pml4_i, pdpt_i, dir_i];
        let mut collections = [pdpt_collection, pd_collection, pt_collection];
        let mut current_table = pml4;

        for (&i, c) in indexes.iter().zip(collections.iter_mut()) {
            current_table = Self::get_next_page_table(i, current_table, c)
        }

        Self::set_addr_to_pt(current_table, table_i, p.start_address());
    }

    fn get_next_page_table<'a>(
        i: PageTableIndex,
        table: &'a mut PageTable,
        collection: &'a mut BTreeMap<PhysAddr, KpBox<PageTable>>,
    ) -> &'a mut KpBox<PageTable> {
        let next_table_a = Self::get_next_page_table_addr_or_create(i, table, collection);

        collection.get_mut(&next_table_a).expect("No such table.")
    }

    fn get_next_page_table_addr_or_create(
        i: PageTableIndex,
        table: &mut PageTable,
        collection: &mut BTreeMap<PhysAddr, KpBox<PageTable>>,
    ) -> PhysAddr {
        if table[i].is_unused() {
            Self::map_to_new_page_table_and_get_addr(i, table, collection)
        } else {
            table[i].addr()
        }
    }

    fn map_to_new_page_table_and_get_addr(
        i: PageTableIndex,
        table: &mut PageTable,
        collection: &mut BTreeMap<PhysAddr, KpBox<PageTable>>,
    ) -> PhysAddr {
        let next: KpBox<PageTable> = KpBox::default();
        let a = next.phys_addr();
        table[i].set_addr(a, Self::flags());
        collection.insert(a, next);
        a
    }

    fn flags() -> PageTableFlags {
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE
    }

    fn set_addr_to_pt(pt: &mut PageTable, i: PageTableIndex, a: PhysAddr) {
        if pt[i].is_unused() {
            pt[i].set_addr(a, Self::flags());
        } else {
            panic!("Mapping is overlapped.")
        }
    }
}
impl Default for Collection {
    fn default() -> Self {
        Self {
            pml4: Pml4Creator::default().create(),
            pdpt_collection: BTreeMap::default(),
            pd_collection: BTreeMap::default(),
            pt_collection: BTreeMap::default(),
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
