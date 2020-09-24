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

        let base = super::map_pages(phys_base, Size::new(size_of::<T>() * num_elements));

        Self {
            base,
            num_elements,
            _marker: PhantomData,
        }
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
        super::unmap_pages(self.base, Size::new(Self::object_size(self.num_elements)))
    }
}
