// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::mem::{
        allocator::{phys::FRAME_MANAGER, virt},
        paging::pml4::PML4,
    },
    core::{
        convert::TryFrom,
        marker::PhantomData,
        mem::size_of,
        ops::{Deref, DerefMut},
        slice,
    },
    os_units::Size,
    x86_64::{
        structures::paging::{Mapper, Page, PageSize, PageTableFlags, PhysFrame, Size4KiB},
        PhysAddr, VirtAddr,
    },
};

pub struct Accessor<'a, T: 'a> {
    base: VirtAddr,
    num_elements: usize,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: 'a> Accessor<'a, T> {
    pub fn new(phys_base: PhysAddr, offset: usize, num_elements: usize) -> Self {
        let phys_base = phys_base + offset;

        let base = Self::map_pages(phys_base, num_elements);

        Self {
            base,
            num_elements,
            _marker: PhantomData,
        }
    }

    fn map_pages(start: PhysAddr, num_elements: usize) -> VirtAddr {
        super::map_pages(start, Size::new(size_of::<T>() * num_elements))
    }

    fn object_size(num_elements: usize) -> usize {
        size_of::<T>() * num_elements
    }
}

impl<'a, T: 'a> Deref for Accessor<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.base.as_ptr(), self.num_elements) }
    }
}

impl<'a, T: 'a> DerefMut for Accessor<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.base.as_mut_ptr(), self.num_elements) }
    }
}

impl<'a, T: 'a> Drop for Accessor<'a, T> {
    fn drop(&mut self) {
        let start_frame_addr = self.base.align_down(Size4KiB::SIZE);
        let end_frame_addr =
            (self.base + Self::object_size(self.num_elements)).align_down(Size4KiB::SIZE);

        let num_pages = Size::new(usize::try_from(end_frame_addr - start_frame_addr).unwrap())
            .as_num_of_pages::<Size4KiB>();

        for i in 0..num_pages.as_usize() {
            let page =
                Page::<Size4KiB>::containing_address(start_frame_addr + Size4KiB::SIZE * i as u64);

            let (_, flush) = PML4.lock().unmap(page).unwrap();
            flush.flush();
        }
    }
}
