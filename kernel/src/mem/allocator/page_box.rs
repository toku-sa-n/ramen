// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{super::paging::pml4::PML4, phys::FRAME_MANAGER, virt},
    core::{
        convert::TryFrom,
        marker::PhantomData,
        mem,
        ops::{Deref, DerefMut},
        slice,
    },
    os_units::{Bytes, NumOfPages},
    x86_64::{
        structures::paging::{
            FrameDeallocator, Mapper, Page, PageSize, PageTableFlags, PhysFrame, Size4KiB,
        },
        VirtAddr,
    },
};

pub struct PageBox<T: ?Sized> {
    virt: VirtAddr,
    bytes: Bytes,
    _marker: PhantomData<T>,
}
impl<T> PageBox<[T]> {
    fn new_slice(num_of_elements: usize) -> Self {
        let bytes = Bytes::new(mem::size_of::<T>() * num_of_elements);
        let virt = Self::allocate_pages(bytes.as_num_of_pages());

        Self {
            virt,
            bytes,
            _marker: PhantomData::<[T]>,
        }
    }

    fn num_of_elements(&self) -> usize {
        self.bytes.as_usize() / mem::size_of::<T>()
    }
}
impl<T> Deref for PageBox<[T]> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.virt.as_ptr(), self.num_of_elements()) }
    }
}
impl<T> DerefMut for PageBox<[T]> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.virt.as_mut_ptr(), self.num_of_elements()) }
    }
}
impl<T: ?Sized> PageBox<T> {
    fn allocate_pages(num_of_pages: NumOfPages<Size4KiB>) -> VirtAddr {
        let virt_addr =
            virt::search_free_addr(num_of_pages).expect("OOM during creating `PageBox`");

        let phys_addr = FRAME_MANAGER
            .lock()
            .alloc(num_of_pages)
            .expect("OOM during creating `PageBox");

        for i in 0..u64::try_from(num_of_pages.as_usize()).unwrap() {
            let page =
                Page::<Size4KiB>::from_start_address(virt_addr + Size4KiB::SIZE * i).unwrap();
            let frame = PhysFrame::from_start_address(phys_addr + Size4KiB::SIZE * i).unwrap();

            unsafe {
                PML4.lock()
                    .map_to(
                        page,
                        frame,
                        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                        &mut *FRAME_MANAGER.lock(),
                    )
                    .unwrap()
                    .flush()
            }
        }

        virt_addr
    }
}
impl<T: ?Sized> Drop for PageBox<T> {
    fn drop(&mut self) {
        let num_of_pages = self.bytes.as_num_of_pages::<Size4KiB>();

        for i in 0..u64::try_from(num_of_pages.as_usize()).unwrap() {
            let page = Page::from_start_address(self.virt + Size4KiB::SIZE * i).unwrap();

            let (frame, flush) = PML4.lock().unmap(page).unwrap();
            flush.flush();
            unsafe { FRAME_MANAGER.lock().deallocate_frame(frame) }
        }
    }
}
