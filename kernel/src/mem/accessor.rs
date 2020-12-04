// SPDX-License-Identifier: GPL-3.0-or-later

use core::{marker::PhantomData, mem, ptr};
use os_units::Bytes;
use x86_64::{PhysAddr, VirtAddr};

pub struct Accessor<T: ?Sized> {
    virt: VirtAddr,
    bytes: Bytes, // The size of `T` is not always computable. Thus save the bytes of objects.
    _marker: PhantomData<T>,
}
impl<T> Accessor<T> {
    /// Safety: This method is unsafe because it can create multiple mutable references to the same
    /// object.
    pub unsafe fn new(phys_base: PhysAddr, offset: Bytes) -> Self {
        let phys_base = phys_base + offset.as_usize();
        let bytes = Bytes::new(mem::size_of::<T>());
        let virt = super::map_pages(phys_base, bytes);

        Self {
            virt,
            bytes,
            _marker: PhantomData,
        }
    }

    pub fn read(&self) -> T {
        unsafe { ptr::read_volatile(self.virt.as_ptr()) }
    }

    pub fn write(&mut self, val: T) {
        unsafe { ptr::write_volatile(self.virt.as_mut_ptr(), val) }
    }

    pub fn update<U>(&mut self, f: U)
    where
        U: Fn(&mut T),
    {
        let mut val = self.read();
        f(&mut val);
        self.write(val);
    }
}

impl<T> Accessor<[T]> {
    /// Safety: This method is unsafe because it can create multiple mutable references to the same
    /// object.
    pub fn new_slice(phys_base: PhysAddr, offset: Bytes, len: usize) -> Self {
        let phys_base = phys_base + offset.as_usize();
        let bytes = Bytes::new(mem::size_of::<T>() * len);
        let virt = super::map_pages(phys_base, bytes);

        Self {
            virt,
            bytes,
            _marker: PhantomData,
        }
    }

    pub fn read(&self, index: usize) -> T {
        assert!(index < self.len());
        unsafe { ptr::read_volatile(self.addr_to_elem(index).as_ptr()) }
    }

    pub fn write(&mut self, index: usize, val: T) {
        assert!(index < self.len());
        unsafe { ptr::write_volatile(self.addr_to_elem(index).as_mut_ptr(), val) }
    }

    pub fn update<U>(&mut self, index: usize, f: U)
    where
        U: Fn(&mut T),
    {
        let mut val = self.read(index);
        f(&mut val);
        self.write(index, val);
    }

    fn addr_to_elem(&self, index: usize) -> VirtAddr {
        self.virt + mem::size_of::<T>() * index
    }

    fn len(&self) -> usize {
        self.bytes.as_usize() / mem::size_of::<T>()
    }
}

impl<T: ?Sized> Accessor<T> {
    /// Safety: This method is unsafe because it can create multiple mutable references to the same
    /// object.
    pub unsafe fn new_from_bytes(phys_base: PhysAddr, offset: Bytes, bytes: Bytes) -> Self {
        let phys_base = phys_base + offset.as_usize();
        let virt = super::map_pages(phys_base, bytes);

        Self {
            virt,
            bytes,
            _marker: PhantomData,
        }
    }

    pub fn virt_addr(&self) -> VirtAddr {
        self.virt
    }
}
impl<T: ?Sized> Drop for Accessor<T> {
    fn drop(&mut self) {
        super::unmap_pages(self.virt, self.bytes);
    }
}
