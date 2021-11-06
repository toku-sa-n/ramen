// SPDX-License-Identifier: GPL-3.0-or-later

use {
    accessor::single::ReadWrite,
    core::{convert::TryInto, num::NonZeroUsize},
    os_units::Bytes,
    x86_64::{PhysAddr, VirtAddr},
};

pub type Single<T> = ReadWrite<T, Mapper>;

/// # Safety
///
/// - `phys_base` must be correct.
/// - The caller must ensure that the returned accessor is the only thing to access the memory address.
///
/// # Panics
///
/// This method panics if `phys_base` is not aligned as the type `T` requires.
#[must_use]
pub unsafe fn single<T>(phys_base: PhysAddr) -> Single<T>
where
    T: Copy,
{
    Single::new(phys_base.as_u64().try_into().unwrap(), Mapper)
}

#[derive(Copy, Clone, Debug)]
pub struct Mapper;
impl accessor::Mapper for Mapper {
    unsafe fn map(&mut self, phys_start: usize, bytes: usize) -> NonZeroUsize {
        let phys_start = PhysAddr::new(phys_start.try_into().unwrap());
        let bytes = Bytes::new(bytes);

        let a = syscalls::map_pages(phys_start, bytes);

        NonZeroUsize::new(a.as_u64().try_into().unwrap()).expect("Failed to map pages.")
    }

    fn unmap(&mut self, virt_start: usize, bytes: usize) {
        let virt_start = VirtAddr::new(virt_start.try_into().unwrap());
        let bytes = Bytes::new(bytes);

        syscalls::unmap_pages(virt_start, bytes);
    }
}
