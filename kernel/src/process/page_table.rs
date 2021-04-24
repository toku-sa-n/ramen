// SPDX-License-Identifier: GPL-3.0-or-later

use crate::mem::{allocator::kpbox::KpBox, paging::pml4::PML4};
use alloc::collections::BTreeMap;
use core::{
    convert::{TryFrom, TryInto},
    ptr,
};
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

    pub(super) fn map_elf(&mut self, name: &str) -> (KpBox<[u8]>, VirtAddr) {
        ElfMapper::new(name, self).map()
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
        Self::set_addr_to_pt(table, i, a);
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

struct ElfMapper<'a> {
    c: &'a mut Collection,
    raw: KpBox<[u8]>,
    output: KpBox<[u8]>,
}
impl<'a> ElfMapper<'a> {
    fn new(name: &str, c: &'a mut Collection) -> Self {
        let handler = crate::fs::get_handler(name);
        let raw = handler.content();
        let raw = KpBox::from(raw);

        let output_bytes = Self::memory_size(&raw);

        Self {
            c,
            raw,
            output: KpBox::new_slice(0_u8, output_bytes),
        }
    }

    fn map(mut self) -> (KpBox<[u8]>, VirtAddr) {
        let raw = self.raw.clone();

        let elf_file = ElfFile::new(&raw);
        let elf_file = elf_file.expect("Not an ELF file.");

        let entry = elf_file.header.pt2.entry_point();
        let entry = VirtAddr::new(entry);

        for ph in elf_file.program_iter() {
            let is_gnu_stack = ph.virtual_addr() == 0;
            if !is_gnu_stack {
                self.map_with_ph(&ph);
            }
        }

        (self.output, entry)
    }

    fn map_with_ph(&mut self, ph: &ProgramHeader<'_>) {
        self.copy_from_raw_to_output(ph);
        self.add_mapping_to_collection(ph);
    }

    fn copy_from_raw_to_output(&mut self, ph: &ProgramHeader<'_>) {
        let src = self.raw.virt_addr() + ph.offset();

        let dst = ph.virtual_addr() - self.elf_bottom().as_u64() + self.output.virt_addr().as_u64();
        let dst = VirtAddr::new(dst);

        let count: usize = ph.file_size().try_into().unwrap();

        unsafe { ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr::<u8>(), count) }
    }

    fn add_mapping_to_collection(&mut self, ph: &ProgramHeader<'_>) {
        let offset = ph.virtual_addr() - self.elf_bottom().as_u64();

        let v = ph.virtual_addr();
        let v = VirtAddr::new(v);

        let p = self.output.phys_addr().as_u64() + offset;
        let p = PhysAddr::new(p);

        let bytes: usize = ph.mem_size().try_into().unwrap();
        let bytes = Bytes::new(bytes);
        let num_of_pages: NumOfPages<Size4KiB> = bytes.as_num_of_pages();

        for i in 0..num_of_pages.as_usize() {
            let offset = NumOfPages::<Size4KiB>::new(i).as_bytes().as_usize();
            let v = Page::from_start_address(v + offset);
            let v = v.expect("Page is not aligned.");

            let p = PhysFrame::from_start_address(p + offset);
            let p = p.expect("PhysFrame is not aligned.");

            self.c.map(v, p);
        }
    }

    fn elf_bottom(&self) -> VirtAddr {
        let elf_file = ElfFile::new(&self.raw);
        let elf_file = elf_file.expect("Not an ELF file.");

        let first_section = elf_file.program_header(0);
        let first_section = first_section.expect("Section header not found.");

        VirtAddr::new(first_section.virtual_addr())
    }

    fn memory_size(elf_raw: &[u8]) -> usize {
        let elf = ElfFile::new(elf_raw);
        let elf = elf.expect("Not an ELF file.");

        let first_section = elf.program_header(0);
        let first_section = first_section.expect("Section header not found.");

        let elf_bottom = first_section.virtual_addr();
        let elf_bottom = VirtAddr::new(elf_bottom);

        let elf_top = elf
            .program_iter()
            .fold(0, |x, a| x.max(a.virtual_addr() + a.mem_size()));
        let elf_top = VirtAddr::new(elf_top);

        usize::try_from(elf_top - elf_bottom).unwrap()
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
