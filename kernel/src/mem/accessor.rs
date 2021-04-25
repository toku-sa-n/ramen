// SPDX-License-Identifier: GPL-3.0-or-later

use core::{convert::TryInto, num::NonZeroUsize};
use os_units::Bytes;
use x86_64::{PhysAddr, VirtAddr};

pub(crate) type Single<T> = accessor::Single<T, Mapper>;

pub(crate) unsafe fn new<T>(phys_base: PhysAddr) -> Single<T>
where
    T: Copy,
{
    accessor::Single::new(phys_base.as_u64().try_into().unwrap(), Mapper)
}

#[derive(Copy, Clone)]
pub(crate) struct Mapper;
impl accessor::Mapper for Mapper {
    unsafe fn map(&mut self, phys_start: usize, bytes: usize) -> NonZeroUsize {
        let phys_start = PhysAddr::new(phys_start.try_into().unwrap());
        let bytes = Bytes::new(bytes);

        let v = super::map_pages(phys_start, bytes);
        let v: usize = v.as_u64().try_into().unwrap();

        NonZeroUsize::new(v).expect("Failed to map pages.")
    }

    fn unmap(&mut self, virt_start: usize, bytes: usize) {
        super::unmap_pages(
            VirtAddr::new(virt_start.try_into().unwrap()),
            Bytes::new(bytes),
        )
    }
}
