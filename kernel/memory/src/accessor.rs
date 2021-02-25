// SPDX-License-Identifier: GPL-3.0-or-later

use accessor::mapper::Mapper;
use core::{convert::TryInto, num::NonZeroUsize};
use os_units::Bytes;
use x86_64::{PhysAddr, VirtAddr};

pub type Single<T> = accessor::Single<T, Mappers>;

/// # Safety
///
/// Refer to the safety note of the `Accessor::Single`.
pub unsafe fn new<T>(phys_base: PhysAddr) -> Single<T>
where
    T: Copy,
{
    accessor::Single::new(phys_base.as_u64().try_into().unwrap(), Mappers::new())
}

#[derive(Copy, Clone)]
pub struct Mappers {
    mapper: fn(PhysAddr, Bytes) -> VirtAddr,
    unmapper: fn(VirtAddr, Bytes),
}
impl Mappers {
    pub fn new() -> Self {
        Self {
            mapper: super::map_pages,
            unmapper: super::unmap_pages,
        }
    }
}
impl Default for Mappers {
    fn default() -> Self {
        Self::new()
    }
}
impl Mapper for Mappers {
    unsafe fn map(&mut self, phys_start: usize, bytes: usize) -> NonZeroUsize {
        NonZeroUsize::new(
            (self.mapper)(
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
        (self.unmapper)(
            VirtAddr::new(virt_start.try_into().unwrap()),
            Bytes::new(bytes),
        )
    }
}
