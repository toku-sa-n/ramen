// SPDX-License-Identifier: GPL-3.0-or-later

use accessor::{error::Error, mapper::Mapper};
use core::{convert::TryInto, num::NonZeroUsize};
use os_units::Bytes;
use x86_64::{PhysAddr, VirtAddr};

pub type Single<T> = accessor::Single<T, Mappers>;
pub type Array<T> = accessor::Array<T, Mappers>;

pub unsafe fn user<T>(phys_base: PhysAddr) -> Result<Single<T>, Error>
where
    T: Copy,
{
    accessor::Single::new(phys_base.as_u64().try_into().unwrap(), Mappers::user())
}

pub unsafe fn user_array<T>(phys_base: PhysAddr, len: usize) -> Result<Array<T>, Error>
where
    T: Copy,
{
    accessor::Array::new(phys_base.as_u64().try_into().unwrap(), len, Mappers::user())
}

pub unsafe fn kernel<T>(phys_base: PhysAddr) -> Result<Single<T>, Error>
where
    T: Copy,
{
    accessor::Single::new(phys_base.as_u64().try_into().unwrap(), Mappers::kernel())
}

#[derive(Copy, Clone)]
pub struct Mappers {
    mapper: fn(PhysAddr, Bytes) -> VirtAddr,
    unmapper: fn(VirtAddr, Bytes),
}
impl Mappers {
    fn kernel() -> Self {
        Self {
            mapper: super::map_pages,
            unmapper: super::unmap_pages,
        }
    }

    pub fn user() -> Self {
        Self {
            mapper: syscalls::map_pages,
            unmapper: syscalls::unmap_pages,
        }
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
