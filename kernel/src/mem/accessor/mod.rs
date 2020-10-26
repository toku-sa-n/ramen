// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::mem::{
        allocator::{phys::FRAME_MANAGER, virt},
        paging::pml4::PML4,
    },
    core::{
        convert::TryFrom,
        marker::PhantomData,
        mem,
        ops::{Deref, DerefMut},
    },
    os_units::Bytes,
    x86_64::{
        structures::paging::{Mapper, Page, PageSize, PageTableFlags, PhysFrame, Size4KiB},
        PhysAddr, VirtAddr,
    },
};

pub mod single_object;
pub mod slice;

struct Accessor<T: ?Sized> {
    virt: VirtAddr,
    bytes: Bytes, // The size of `T` is not always computable. Thus save the bytes of objects.
    _marker: PhantomData<T>,
}
impl<T> Accessor<T> {
    fn new(phys_base: PhysAddr, offset: Bytes) -> Self {
        let phys_base = phys_base + offset.as_usize();
        let virt = map_pages(phys_base, Bytes::new(mem::size_of::<T>()));

        Self {
            virt,
            bytes: Bytes::new(mem::size_of::<T>()),
            _marker: PhantomData,
        }
    }
}
impl<T> Deref for Accessor<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.virt.as_ptr() }
    }
}
impl<T> DerefMut for Accessor<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.virt.as_mut_ptr() }
    }
}

impl<T> Accessor<[T]> {
    fn new(phys_base: PhysAddr, offset: Bytes, len: usize) -> Self {
        let phys_base = phys_base + offset.as_usize();
        let virt = map_pages(phys_base, Bytes::new(mem::size_of::<T>() * len));

        Self {
            virt,
            bytes: Bytes::new(mem::size_of::<T>() * len),
            _marker: PhantomData,
        }
    }

    fn len(&self) -> usize {
        self.bytes.as_usize() / mem::size_of::<T>()
    }
}
impl<T> Deref for Accessor<[T]> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.virt.as_ptr(), self.len()) }
    }
}
impl<T> DerefMut for Accessor<[T]> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::slice::from_raw_parts_mut(self.virt.as_mut_ptr(), self.len()) }
    }
}

impl<T: ?Sized> Drop for Accessor<T> {
    fn drop(&mut self) {
        unmap_pages(self.virt, self.bytes)
    }
}

fn map_pages(start: PhysAddr, object_size: Bytes) -> VirtAddr {
    let start_frame_addr = start.align_down(Size4KiB::SIZE);
    let end_frame_addr = (start + object_size.as_usize()).align_down(Size4KiB::SIZE);

    let num_pages = Bytes::new(usize::try_from(end_frame_addr - start_frame_addr).unwrap() + 1)
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

    let page_offset = start.as_u64() % Size4KiB::SIZE;

    virt + page_offset
}

fn unmap_pages(start: VirtAddr, object_size: Bytes) {
    let start_frame_addr = start.align_down(Size4KiB::SIZE);
    let end_frame_addr = (start + object_size.as_usize()).align_down(Size4KiB::SIZE);

    let num_pages = Bytes::new(usize::try_from(end_frame_addr - start_frame_addr).unwrap())
        .as_num_of_pages::<Size4KiB>();

    for i in 0..num_pages.as_usize() {
        let page =
            Page::<Size4KiB>::containing_address(start_frame_addr + Size4KiB::SIZE * i as u64);

        let (_, flush) = PML4.lock().unmap(page).unwrap();
        flush.flush();
    }
}
