// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::mem::{
        allocator::{phys::FRAME_MANAGER, virt},
        paging::pml4::PML4,
    },
    core::convert::TryFrom,
    os_units::{Bytes, Size},
    x86_64::{
        structures::paging::{Mapper, Page, PageSize, PageTableFlags, PhysFrame, Size4KiB},
        PhysAddr, VirtAddr,
    },
};

pub mod single_object;
pub mod slice;

trait Accessor {
    fn map_pages(start: PhysAddr, object_size: Size<Bytes>) -> VirtAddr {
        let start_frame_addr = start.align_down(Size4KiB::SIZE);
        let end_frame_addr = (start + object_size.as_usize()).align_down(Size4KiB::SIZE);

        let num_pages = Size::new(usize::try_from(end_frame_addr - start_frame_addr).unwrap() + 1)
            .as_num_of_pages::<Size4KiB>();

        let virt = virt::search_free_addr(num_pages)
            .expect("OOM during creating a new accessor to a register.");

        for i in 0..num_pages.as_usize() {
            let page = Page::<Size4KiB>::containing_address(virt + Size4KiB::SIZE * i as u64);
            let frame = PhysFrame::containing_address(start_frame_addr + Size4KiB::SIZE * i as u64);

            unsafe {
                PML4.lock()
                    .map_to(
                        page,
                        frame,
                        PageTableFlags::PRESENT,
                        &mut *FRAME_MANAGER.lock(),
                    )
                    .unwrap()
                    .flush()
            }
        }

        virt
    }
}
