// SPDX-License-Identifier: GPL-3.0-or-later

use core::{marker::PhantomData, mem, ptr};
use os_units::Bytes;
use x86_64::{PhysAddr, VirtAddr};

pub struct Accessor<T: ?Sized> {
    virt: VirtAddr,
    bytes: Bytes, // The size of `T` is not always computable. Thus save the bytes of objects.
    _marker: PhantomData<T>,
    mappers: Mappers,
}
impl<T> Accessor<T> {
    /// SAFETY: This method is unsafe because it can create multiple mutable references to the same
    /// object.
    pub unsafe fn kernel(phys_base: PhysAddr, offset: Bytes) -> Self {
        Self::new(&EffectiveAddr::new(phys_base, offset), Mappers::kernel())
    }

    /// SAFETY: This method is unsafe because it can create multiple mutable references to the same
    /// object.
    pub unsafe fn user(phys_base: PhysAddr, offset: Bytes) -> Self {
        Self::new(&EffectiveAddr::new(phys_base, offset), Mappers::user())
    }

    /// SAFETY: This method is unsafe because it can create multiple mutable references to the same
    /// object.
    unsafe fn new(effective: &EffectiveAddr, mappers: Mappers) -> Self {
        let phys_base = effective.calculate();
        let bytes = Bytes::new(mem::size_of::<T>());
        let virt = mappers.map(phys_base, bytes);

        Self {
            virt,
            bytes,
            _marker: PhantomData,
            mappers,
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
    /// # Safety: This method is unsafe because it can create multiple mutable references to the
    /// same object.
    pub unsafe fn user_slice(phys_base: PhysAddr, offset: Bytes, len: usize) -> Self {
        Self::new_slice(&EffectiveAddr::new(phys_base, offset), len, Mappers::user())
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

    /// SAFETY: This method is unsafe because it can create multiple mutable references to the same
    /// object.
    fn new_slice(effective: &EffectiveAddr, len: usize, mappers: Mappers) -> Self {
        let phys_base = effective.calculate();
        let bytes = Bytes::new(mem::size_of::<T>() * len);
        let virt = mappers.map(phys_base, bytes);

        Self {
            virt,
            bytes,
            _marker: PhantomData,
            mappers,
        }
    }

    fn addr_to_elem(&self, index: usize) -> VirtAddr {
        self.virt + mem::size_of::<T>() * index
    }

    fn len(&self) -> usize {
        self.bytes.as_usize() / mem::size_of::<T>()
    }
}

impl<T: ?Sized> Drop for Accessor<T> {
    fn drop(&mut self) {
        self.mappers.unmap(self.virt, self.bytes);
    }
}

struct EffectiveAddr {
    base: PhysAddr,
    offset: Bytes,
}
impl EffectiveAddr {
    fn new(base: PhysAddr, offset: Bytes) -> Self {
        Self { base, offset }
    }

    fn calculate(&self) -> PhysAddr {
        self.base + self.offset.as_usize()
    }
}

type Mapper = fn(PhysAddr, Bytes) -> VirtAddr;
type Unmapper = fn(VirtAddr, Bytes);

struct Mappers {
    mapper: Mapper,
    unmapper: Unmapper,
}
impl Mappers {
    fn kernel() -> Self {
        Self {
            mapper: super::map_pages,
            unmapper: super::unmap_pages,
        }
    }

    fn user() -> Self {
        Self {
            mapper: syscalls::map_pages,
            unmapper: syscalls::unmap_pages,
        }
    }

    fn map(&self, p: PhysAddr, b: Bytes) -> VirtAddr {
        (self.mapper)(p, b)
    }

    fn unmap(&self, v: VirtAddr, b: Bytes) {
        (self.unmapper)(v, b)
    }
}
