use {
    allocator::virt,
    boot_info::mem::MemoryDescriptor,
    core::convert::TryFrom,
    os_units::Bytes,
    predefined_mmap::{KERNEL_ADDR, STACK_BASE},
    x86_64::{
        structures::paging::{
            page::PageRange, Page, PageSize, PageTableFlags, PhysFrame, Size4KiB,
        },
        PhysAddr, VirtAddr,
    },
};

pub(crate) mod accessor;
pub(crate) mod allocator;
pub(crate) mod elf;
pub(crate) mod paging;

pub(super) fn init(mem_map: &[MemoryDescriptor]) {
    allocator::heap::init();
    allocator::phys::init(mem_map);
    paging::mark_pages_as_unused();
}

pub(super) fn map_pages_for_user(start: PhysAddr, object_size: Bytes) -> VirtAddr {
    map_pages_from(
        start,
        object_size,
        PageRange {
            start: Page::from_start_address(VirtAddr::new(0x1000)).unwrap(),
            end: Page::from_start_address(KERNEL_ADDR).unwrap(),
        },
    )
}

pub(super) fn map_pages_for_kernel(start: PhysAddr, object_size: Bytes) -> VirtAddr {
    map_pages_from(
        start,
        object_size,
        PageRange {
            start: Page::from_start_address(STACK_BASE).unwrap(),
            end: Page::from_start_address(VirtAddr::new(0xffff_ffff_ffff_f000)).unwrap(),
        },
    )
}

pub(super) fn unmap_pages(start: VirtAddr, object_size: Bytes) {
    let start_frame_addr = start.align_down(Size4KiB::SIZE);
    let end_frame_addr = (start + object_size.as_usize()).align_down(Size4KiB::SIZE);

    let num_pages = Bytes::new(usize::try_from(end_frame_addr - start_frame_addr).unwrap())
        .as_num_of_pages::<Size4KiB>();

    for i in 0..num_pages.as_usize() {
        let page =
            Page::<Size4KiB>::containing_address(start_frame_addr + Size4KiB::SIZE * i as u64);

        paging::unmap(page).unwrap();
    }
}

fn map_pages_from(start: PhysAddr, object_size: Bytes, region: PageRange) -> VirtAddr {
    let start_frame_addr = start.align_down(Size4KiB::SIZE);
    let end_frame_addr = (start + object_size.as_usize()).align_down(Size4KiB::SIZE);

    let num_pages = Bytes::new(usize::try_from(end_frame_addr - start_frame_addr).unwrap() + 1)
        .as_num_of_pages::<Size4KiB>();

    let virt = virt::search_free_addr_from(num_pages, region)
        .expect("OOM during creating a new accessor to a register.");

    for i in 0..num_pages.as_usize() {
        let page = Page::<Size4KiB>::containing_address(virt + Size4KiB::SIZE * i as u64);
        let frame = PhysFrame::containing_address(start_frame_addr + Size4KiB::SIZE * i as u64);
        let flag =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

        unsafe {
            paging::map_to(page, frame, flag).unwrap();
        }
    }

    let page_offset = start.as_u64() % Size4KiB::SIZE;

    virt + page_offset
}
