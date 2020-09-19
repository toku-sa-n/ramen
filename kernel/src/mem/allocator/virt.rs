// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::mem::{allocator::phys::FRAME_MANAGER, paging::pml4::PML4},
    common::constant::BYTES_AVAILABLE_RAM,
    core::convert::TryFrom,
    x86_64::{
        structures::paging::{
            FrameDeallocator, Mapper, MapperAllSizes, Page, PageSize, PageTableFlags, PhysFrame,
            Size4KiB,
        },
        PhysAddr, VirtAddr,
    },
};

pub fn map_to_phys_temporary<T, U>(addr: PhysAddr, f: T) -> U
where
    T: Fn(VirtAddr) -> U,
{
    map_temporary(|virt| {
        let page = Page::<Size4KiB>::containing_address(virt);
        let frame = PhysFrame::containing_address(addr);
        unsafe {
            PML4.lock()
                .map_to(
                    page,
                    frame,
                    PageTableFlags::PRESENT,
                    &mut *FRAME_MANAGER.lock(),
                )
                .expect("OOM during `map_to_phys_temporary")
                .flush()
        };

        let return_value = f(virt);

        let (frame, flush) = PML4.lock().unmap(page).unwrap();
        flush.flush();

        unsafe { FRAME_MANAGER.lock().deallocate_frame(frame) }

        return_value
    })
}

pub fn search_first_unused_page() -> Option<Page> {
    for addr in
        (0..BYTES_AVAILABLE_RAM.as_usize()).step_by(usize::try_from(Size4KiB::SIZE).unwrap())
    {
        let virt_addr = VirtAddr::new(addr as _);
        if PML4.lock().translate_addr(virt_addr).is_none() {
            return Some(Page::containing_address(virt_addr));
        }
    }
    None
}

fn map_temporary<T, U>(f: T) -> U
where
    T: Fn(VirtAddr) -> U,
{
    match search_first_unused_page() {
        Some(addr) => f(addr.start_address()),
        None => panic!("OOM during `map_temporary`"),
    }
}
