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
        NonZeroUsize::new(
            super::map_pages(
                PhysAddr::new(phys_start.try_into().unwrap()),
                Bytes::new(bytes),
            )
            .as_u64()
            .try_into()
            .unwrap(),
        )
        .unwrap()
    }

    fn unmap(&mut self, virt_start: usize, bytes: usize) {
        super::unmap_pages(
            VirtAddr::new(virt_start.try_into().unwrap()),
            Bytes::new(bytes),
        )
    }
}
