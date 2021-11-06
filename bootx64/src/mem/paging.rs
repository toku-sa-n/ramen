// SPDX-License-Identifier: GPL-3.0-or-later

use {
    boot_info::mem::MemoryDescriptor,
    common::mem::reserved,
    core::convert::TryFrom,
    predefined_mmap::RECUR_PML4_ADDR,
    x86_64::{
        addr::PhysAddr,
        registers::control::{Cr0, Cr0Flags, Cr3},
        structures::paging::{
            FrameAllocator, Mapper, Page, PageSize, PageTable, PageTableFlags, PhysFrame,
            RecursivePageTable, Size4KiB,
        },
    },
};

struct AllocatorWithEfiMemoryMap<'a> {
    mem_map: &'a mut [MemoryDescriptor],
}

impl<'a> AllocatorWithEfiMemoryMap<'a> {
    fn new(mem_map: &'a mut [MemoryDescriptor]) -> Self {
        Self { mem_map }
    }
}

unsafe impl FrameAllocator<Size4KiB> for AllocatorWithEfiMemoryMap<'_> {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        for descriptor in self.mem_map.iter_mut() {
            if descriptor.num_pages.as_usize() > 0 {
                let addr = descriptor.start;
                descriptor.start += Size4KiB::SIZE;
                descriptor.num_pages -= 1;

                return Some(PhysFrame::containing_address(addr));
            }
        }

        None
    }
}

pub fn init(boot_info: &mut boot_info::Info, reserved: &reserved::Map) {
    remove_table_protection();

    enable_recursive_mapping();

    let mut allocator = AllocatorWithEfiMemoryMap::new(boot_info.mem_map_mut());

    for region in reserved.iter() {
        map_virt_to_phys(region, &mut allocator);
    }
}

fn enable_recursive_mapping() {
    let p4: &mut PageTable = unsafe { &mut *(get_pml4_addr().as_u64() as *mut _) };

    p4[510].set_addr(
        get_pml4_addr(),
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE,
    );
}

fn remove_table_protection() {
    unsafe {
        Cr0::update(|flags| {
            flags.remove(Cr0Flags::WRITE_PROTECT);
        });
    }
}

fn map_virt_to_phys(region: &reserved::Range, allocator: &mut AllocatorWithEfiMemoryMap<'_>) {
    let p4 = unsafe { &mut *(RECUR_PML4_ADDR.as_mut_ptr()) };
    let mut p4 = RecursivePageTable::new(p4).unwrap();

    let num_of_pages = region.bytes().as_num_of_pages::<Size4KiB>().as_usize();
    for i in 0..num_of_pages {
        let v = Page::<Size4KiB>::containing_address(
            region.virt() + usize::try_from(Size4KiB::SIZE).unwrap() * i,
        );
        let p = PhysFrame::containing_address(
            region.phys() + usize::try_from(Size4KiB::SIZE).unwrap() * i,
        );
        let f =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
        unsafe { p4.map_to(v, p, f, allocator) }.unwrap().flush();
    }
}

fn get_pml4_addr() -> PhysAddr {
    let (frame, _) = Cr3::read();
    frame.start_address()
}
