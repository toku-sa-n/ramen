// SPDX-License-Identifier: GPL-3.0-or-later

use common::constant::*;
use common::size::{Byte, Size};
use common::vram;
use uefi::table::boot;
use uefi::table::boot::{AllocateType, MemoryType};
use x86_64::addr::{PhysAddr, VirtAddr};
use x86_64::registers::control::{Cr0, Cr0Flags};
use x86_64::structures::paging::{
    FrameAllocator, Mapper, Page, PageSize, PageTable, PageTableFlags, PhysFrame,
    RecursivePageTable, Size4KiB,
};

struct PageMapInfo {
    virt: VirtAddr,
    phys: PhysAddr,
    bytes: Size<Byte>,
}

impl PageMapInfo {
    fn new(virt: VirtAddr, phys: PhysAddr, bytes: Size<Byte>) -> Self {
        Self { virt, phys, bytes }
    }

    fn map(&self, allocator: &mut AllocatorWithEfiMemoryMap) -> () {
        map_virt_to_phys(self.virt, self.phys, self.bytes, allocator);
    }
}

struct AllocatorWithEfiMemoryMap<'a> {
    boot_services: &'a boot::BootServices,
}

impl<'a> AllocatorWithEfiMemoryMap<'a> {
    fn new(boot_services: &'a boot::BootServices) -> Self {
        Self { boot_services }
    }
}

unsafe impl<'a> FrameAllocator<Size4KiB> for AllocatorWithEfiMemoryMap<'a> {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        Some(PhysFrame::containing_address(PhysAddr::new(
            self.boot_services
                .allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 1)
                .ok()?
                .unwrap(),
        )))
    }
}

pub fn init(
    boot_services: &boot::BootServices,
    vram: &vram::Info,
    addr_kernel: PhysAddr,
    bytes_kernel: Size<Byte>,
    stack_addr: PhysAddr,
) -> () {
    remove_table_protection();

    enable_recursive_mapping();

    let mut allocator = AllocatorWithEfiMemoryMap::new(boot_services);

    let map_info = [
        PageMapInfo::new(KERNEL_ADDR, addr_kernel, bytes_kernel),
        PageMapInfo::new(
            PML4_ADDR,
            get_pml4_addr(),
            Size::new(Size4KiB::SIZE as usize),
        ),
        PageMapInfo::new(VRAM_ADDR, vram.phys_ptr(), vram.bytes()),
        PageMapInfo::new(
            STACK_BASE - NUM_OF_PAGES_STACK.as_bytes().as_usize(),
            stack_addr,
            NUM_OF_PAGES_STACK.as_bytes(),
        ),
    ];

    for info in &map_info {
        info.map(&mut allocator);
    }
}

fn enable_recursive_mapping() -> () {
    let p4: &mut PageTable = unsafe { &mut *(get_pml4_addr().as_u64() as *mut _) };

    p4[511].set_addr(get_pml4_addr(), PageTableFlags::PRESENT);
}

fn remove_table_protection() -> () {
    unsafe {
        Cr0::update(|flags| {
            flags.remove(Cr0Flags::WRITE_PROTECT);
        })
    }
}

fn map_virt_to_phys(
    virt: VirtAddr,
    phys: PhysAddr,
    bytes: Size<Byte>,
    allocator: &mut AllocatorWithEfiMemoryMap,
) -> () {
    let p4 = unsafe { &mut *(RECUR_PML4_ADDR.as_mut_ptr()) };
    let mut p4 = RecursivePageTable::new(p4).unwrap();

    let num_of_pages = bytes.as_num_of_pages().as_usize();
    for i in 0..num_of_pages {
        unsafe {
            p4.map_to_with_table_flags::<AllocatorWithEfiMemoryMap>(
                Page::<Size4KiB>::containing_address(virt + Size4KiB::SIZE as usize * i),
                PhysFrame::containing_address(phys + Size4KiB::SIZE as usize * i),
                PageTableFlags::PRESENT,
                PageTableFlags::PRESENT,
                allocator,
            )
        }
        .unwrap()
        .flush();
    }
}

fn get_pml4_addr() -> PhysAddr {
    let addr;
    unsafe {
        asm!("mov rax, cr3",out("rax") addr,options(nomem, preserves_flags, nostack));
    }

    PhysAddr::new(addr)
}
