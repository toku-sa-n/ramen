// SPDX-License-Identifier: GPL-3.0-or-later

use {
    accessor::single::ReadWrite,
    core::{convert::TryInto, num::NonZeroUsize},
    os_units::Bytes,
    x86_64::{PhysAddr, VirtAddr},
};

pub(crate) type Single<T> = ReadWrite<T, Mapper>;

pub(crate) unsafe fn new<T>(phys_base: PhysAddr) -> Single<T>
where
    T: Copy,
{
    // SAFETY: The caller must ensure that `phys_base` is the correct address.
    unsafe { ReadWrite::new(phys_base.as_u64().try_into().unwrap(), Mapper) }
}

#[derive(Copy, Clone)]
pub(crate) struct Mapper;
impl accessor::Mapper for Mapper {
    unsafe fn map(&mut self, phys_start: usize, bytes: usize) -> NonZeroUsize {
        let phys_start = PhysAddr::new(phys_start.try_into().unwrap());
        let bytes = Bytes::new(bytes);

        let v = super::map_pages_for_kernel(phys_start, bytes);
        let v: usize = v.as_u64().try_into().unwrap();

        NonZeroUsize::new(v).expect("Failed to map pages.")
    }

    fn unmap(&mut self, virt_start: usize, bytes: usize) {
        let virt_start = VirtAddr::new(virt_start.try_into().unwrap());
        let bytes = Bytes::new(bytes);

        super::unmap_pages(virt_start, bytes);
    }
}
