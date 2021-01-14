// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::paging::pml4::PML4;
use core::{
    convert::TryFrom,
    fmt,
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    ptr, slice,
};
use os_units::{Bytes, NumOfPages};
use x86_64::{
    structures::paging::{Size4KiB, Translate},
    PhysAddr, VirtAddr,
};

pub struct PageBox<T: ?Sized> {
    virt: VirtAddr,
    bytes: Bytes,
    allocator: Allocator,
    _marker: PhantomData<T>,
}
impl<T> PageBox<T> {
    pub fn user(x: T) -> Self {
        Self::new(x, Allocator::user())
    }

    fn new(x: T, a: Allocator) -> Self {
        let bytes = Bytes::new(mem::size_of::<T>());
        let mut page_box = Self::from_bytes(bytes, a);
        page_box.write_initial_value(x);
        page_box
    }

    fn write_initial_value(&mut self, x: T) {
        // SAFETY: This operation is safe because the memory `self.virt.as_mut_ptr` points is
        // allocated, and is page-aligned.
        unsafe {
            ptr::write(self.virt.as_mut_ptr(), x);
        }
    }
}
impl<T> Deref for PageBox<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: This operation is safe because the memory region `virt` points is allocated and
        // is not used by the others.
        unsafe { &*self.virt.as_ptr() }
    }
}
impl<T> DerefMut for PageBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: This operation is safe because the memory region `virt` points is allocated and
        // is not used by the others.
        unsafe { &mut *self.virt.as_mut_ptr() }
    }
}
impl<T: fmt::Display> fmt::Display for PageBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}
impl<T> fmt::Debug for PageBox<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T> PageBox<[T]>
where
    T: Copy + Clone,
{
    pub fn user_slice(x: T, num_of_elements: usize) -> Self {
        Self::new_slice(x, num_of_elements, Allocator::user())
    }

    fn new_slice(x: T, num_of_elements: usize, a: Allocator) -> Self {
        let bytes = Bytes::new(mem::size_of::<T>() * num_of_elements);
        let mut page_box = Self::from_bytes(bytes, a);
        page_box.write_all_elements_with_same_value(x);
        page_box
    }

    fn write_all_elements_with_same_value(&mut self, x: T)
    where
        T: Copy + Clone,
    {
        for i in 0..self.len() {
            let ptr: usize = usize::try_from(self.virt.as_u64()).unwrap() + mem::size_of::<T>() * i;

            // SAFETY: This operation is safe. The memory ptr points is allocated and is aligned
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
impl<T> fmt::Display for PageBox<[T]>
where
    T: Copy + Clone,
    [T]: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}
impl<T> fmt::Debug for PageBox<[T]>
where
    T: Copy + Clone,
    [T]: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized> PageBox<T> {
    pub fn virt_addr(&self) -> VirtAddr {
        self.virt
    }

    pub fn phys_addr(&self) -> PhysAddr {
        PML4.lock().translate_addr(self.virt).unwrap()
    }

    pub fn bytes(&self) -> Bytes {
        self.bytes
    }

    fn from_bytes(bytes: Bytes, allocator: Allocator) -> Self {
        let virt =
            (allocator.alloc)(bytes.as_num_of_pages()).expect("OOM during creating a new page box");

        let mut page_box = Self {
            virt,
            bytes,
            allocator,
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
}
impl<T: ?Sized> Drop for PageBox<T> {
    fn drop(&mut self) {
        let num_of_pages = self.bytes.as_num_of_pages::<Size4KiB>();
        (self.allocator.dealloc)(self.virt, num_of_pages);
    }
}

struct Allocator {
    alloc: fn(NumOfPages<Size4KiB>) -> Option<VirtAddr>,
    dealloc: fn(VirtAddr, NumOfPages<Size4KiB>),
}
impl Allocator {
    fn user() -> Self {
        Self {
            alloc: Self::syscalls_allocate_pages,
            dealloc: syscalls::deallocate_pages,
        }
    }

    fn syscalls_allocate_pages(pages: NumOfPages<Size4KiB>) -> Option<VirtAddr> {
        let a = syscalls::allocate_pages(pages);
        if a.is_null() {
            None
        } else {
            Some(a)
        }
    }
}
