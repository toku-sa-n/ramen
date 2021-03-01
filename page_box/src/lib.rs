// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]

use core::{
    convert::TryFrom,
    fmt,
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    ptr, slice,
};
use os_units::Bytes;
use x86_64::{structures::paging::Size4KiB, PhysAddr, VirtAddr};

pub struct PageBox<T: ?Sized> {
    virt: VirtAddr,
    phys: PhysAddr,
    bytes: Bytes,
    _marker: PhantomData<T>,
}
impl<T> PageBox<T> {
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
impl<T> Default for PageBox<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::from(T::default())
    }
}
impl<T> Clone for PageBox<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self::from((&**self).clone())
    }
}
impl<T> From<T> for PageBox<T> {
    fn from(x: T) -> Self {
        let bytes = Bytes::new(mem::size_of::<T>());
        let mut page_box = Self::from_bytes(bytes);
        page_box.write_initial_value(x);
        page_box
    }
}

impl<T> PageBox<[T]>
where
    T: Clone,
{
    pub fn new_slice(x: T, num_of_elements: usize) -> Self {
        let bytes = Bytes::new(mem::size_of::<T>() * num_of_elements);
        let mut page_box = Self::from_bytes(bytes);
        page_box.write_all_elements_with_same_value(x);
        page_box
    }

    fn write_all_elements_with_same_value(&mut self, x: T) {
        for i in 0..self.len() {
            let ptr: usize = usize::try_from(self.virt.as_u64()).unwrap() + mem::size_of::<T>() * i;

            // SAFETY: This operation is safe. The memory ptr points is allocated and is aligned
            // because the first elements is page-aligned.
            unsafe { ptr::write(ptr as *mut T, x.clone()) }
        }
    }

    fn num_of_elements(&self) -> usize {
        self.bytes.as_usize() / mem::size_of::<T>()
    }
}
impl<T> Deref for PageBox<[T]>
where
    T: Clone,
{
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.virt.as_ptr(), self.num_of_elements()) }
    }
}
impl<T> DerefMut for PageBox<[T]>
where
    T: Clone,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.virt.as_mut_ptr(), self.num_of_elements()) }
    }
}
impl<T> fmt::Display for PageBox<[T]>
where
    T: Clone,
    [T]: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}
impl<T> fmt::Debug for PageBox<[T]>
where
    T: Clone,
    [T]: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}
impl<T> Clone for PageBox<[T]>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut b = Self::new_slice(self[0].clone(), self.len());

        for (dst, src) in b.iter_mut().zip(self.iter()) {
            *dst = src.clone();
        }

        b
    }
}
impl<T: Copy> From<&[T]> for PageBox<[T]> {
    fn from(s: &[T]) -> Self {
        let b = Self::new_slice(s[0], s.len());

        unsafe {
            ptr::copy_nonoverlapping(s.as_ptr(), b.virt_addr().as_mut_ptr(), s.len());
        }

        b
    }
}

impl<T: ?Sized> PageBox<T> {
    #[must_use]
    pub fn virt_addr(&self) -> VirtAddr {
        self.virt
    }

    #[must_use]
    pub fn phys_addr(&self) -> PhysAddr {
        self.phys
    }

    #[must_use]
    pub fn bytes(&self) -> Bytes {
        self.bytes
    }

    fn from_bytes(bytes: Bytes) -> Self {
        let virt = syscalls::allocate_pages(bytes.as_num_of_pages());

        if virt.is_null() {
            panic!("Failed to allocate pages.");
        }

        let phys = syscalls::translate_address(virt);

        if phys.is_null() {
            panic!("Address {:?} is not mapped.", virt);
        }

        Self {
            virt,
            phys,
            bytes,
            _marker: PhantomData,
        }
    }
}
impl<T: ?Sized> Drop for PageBox<T> {
    fn drop(&mut self) {
        let num_of_pages = self.bytes.as_num_of_pages::<Size4KiB>();
        syscalls::deallocate_pages(self.virt, num_of_pages);
    }
}
