// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{super::paging::pml4::PML4, phys::FRAME_MANAGER, virt},
    core::{
        convert::TryFrom,
        marker::PhantomData,
        mem,
        ops::{Deref, DerefMut},
        ptr, slice,
    },
    os_units::{Bytes, NumOfPages},
    x86_64::{
        structures::paging::{
            FrameDeallocator, Mapper, MapperAllSizes, Page, PageSize, PageTableFlags, PhysFrame,
            Size4KiB,
        },
        PhysAddr, VirtAddr,
    },
};

pub struct PageBox<T: ?Sized> {
    virt: VirtAddr,
    bytes: Bytes,
    _marker: PhantomData<T>,
}
impl<T> PageBox<T> {
    pub fn new(x: T) -> Self {
        let bytes = Bytes::new(mem::size_of::<T>());
        let mut page_box = Self::new_zeroed_from_bytes(bytes);
        page_box.write_initial_value(x);
        page_box
    }

    fn write_initial_value(&mut self, x: T) {
        // Safety: This operation is safe because the memory `self.virt.as_mut_ptr` points is
        // allocated, and is page-aligned.
        unsafe {
            ptr::write(self.virt.as_mut_ptr(), x);
        }
    }
}
impl<T> Deref for PageBox<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // Safety: This operation is safe because the memory region `virt` points is allocated and
        // is not used by the others.
        unsafe { &*self.virt.as_ptr() }
    }
}
impl<T> DerefMut for PageBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety: This operation is safe because the memory region `virt` points is allocated and
        // is not used by the others.
        unsafe { &mut *self.virt.as_mut_ptr() }
    }
}

impl<T> PageBox<[T]>
where
    T: Copy + Clone,
{
    pub fn new_slice(x: T, num_of_elements: usize) -> Self {
        let bytes = Bytes::new(mem::size_of::<T>() * num_of_elements);
        let mut page_box = Self::new_zeroed_from_bytes(bytes);
        page_box.write_all_elements_with_same_value(x);
        page_box
    }

    fn write_all_elements_with_same_value(&mut self, x: T)
    where
        T: Copy + Clone,
    {
        for i in 0..self.len() {
            let ptr: usize = usize::try_from(self.virt.as_u64()).unwrap() + mem::size_of::<T>() * i;

            // Safety: This operation is safe. The memory ptr points is allocated and is aligned
            // because the first elements is page-aligned.
            unsafe { ptr::write(ptr as *mut T, x) }
        }
    }

    fn num_of_elements(&self) -> usize {
        self.bytes.as_usize() / mem::size_of::<T>()
    }
}
impl<T> Deref for PageBox<[T]>
where
    T: Copy + Clone,
{
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.virt.as_ptr(), self.num_of_elements()) }
    }
}
impl<T> DerefMut for PageBox<[T]>
where
    T: Copy + Clone,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.virt.as_mut_ptr(), self.num_of_elements()) }
    }
}
impl<T: ?Sized> PageBox<T> {
    pub fn phys_addr(&self) -> PhysAddr {
        PML4.lock().translate_addr(self.virt).unwrap()
    }

    fn new_zeroed_from_bytes(bytes: Bytes) -> Self {
        let virt = Self::allocate_pages(bytes.as_num_of_pages());

        let mut page_box = Self {
            virt,
            bytes,
            _marker: PhantomData,
        };
        page_box.write_all_bytes_with_zero();
        page_box
    }

    fn write_all_bytes_with_zero(&mut self) {
        unsafe {
            core::ptr::write_bytes(self.virt.as_mut_ptr::<u8>(), 0, self.bytes.as_usize());
        }
    }

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
