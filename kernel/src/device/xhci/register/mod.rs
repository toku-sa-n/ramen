// SPDX-License-Identifier: GPL-3.0-or-later

pub mod hc_capability_registers;
pub mod hc_operational_registers;
pub mod usb_legacy_support_capability;

use {
    crate::mem::{
        allocator::{phys::FRAME_MANAGER, virt},
        paging::pml4::PML4,
    },
    core::{
        marker::PhantomData,
        mem::size_of,
        ops::{Deref, DerefMut},
    },
    os_units::Size,
    x86_64::{
        structures::paging::{Mapper, Page, PageSize, PageTableFlags, PhysFrame, Size4KiB},
        PhysAddr, VirtAddr,
    },
};

pub trait Register {}

pub struct Accessor<'a, T: 'a + Register> {
    base: VirtAddr,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: 'a + Register> Accessor<'a, T> {
    pub fn new(phys_base: PhysAddr, offset: usize) -> Self {
        let phys_base = phys_base + offset;

        let base = Self::map_pages(phys_base);

        Self {
            base,
            _marker: PhantomData,
        }
    }

    fn map_pages(start: PhysAddr) -> VirtAddr {
        let start_frame_addr = start.align_down(Size4KiB::SIZE);
        let end_frame_addr = (start + size_of::<T>()).align_down(Size4KiB::SIZE);

        let num_pages = Size::new((end_frame_addr - start_frame_addr) as usize + 1)
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

impl<'a, T: 'a + Register> Deref for Accessor<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.base.as_ptr() }
    }
}

impl<'a, T: 'a + Register> DerefMut for Accessor<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.base.as_mut_ptr() }
    }
}

impl<'a, T: 'a + Register> Drop for Accessor<'a, T> {
    fn drop(&mut self) {
        let start_frame_addr = self.base.align_down(Size4KiB::SIZE);
        let end_frame_addr = (self.base + size_of::<T>()).align_down(Size4KiB::SIZE);

        let num_pages =
            Size::new((end_frame_addr - start_frame_addr) as _).as_num_of_pages::<Size4KiB>();

        for i in 0..num_pages.as_usize() {
            let page =
                Page::<Size4KiB>::containing_address(start_frame_addr + Size4KiB::SIZE * i as u64);

            let (_, flush) = PML4.lock().unmap(page).unwrap();
            flush.flush();
        }
    }
}
