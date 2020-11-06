// SPDX-License-Identifier: GPL-3.0-or-later

use {crate::mem::accessor::Accessor, core::convert::TryInto, os_units::Bytes, x86_64::PhysAddr};

const NUM_OF_REGISTERS: usize = 256;

pub struct Array(Accessor<[u32]>);
impl Array {
    pub fn new(mmio_base: PhysAddr, db_off: u32) -> Self {
        Self(Accessor::new_slice(
            mmio_base,
            Bytes::new(db_off.try_into().unwrap()),
            NUM_OF_REGISTERS,
        ))
    }

    pub fn update<T>(&mut self, index: usize, f: T)
    where
        T: Fn(&mut u32),
    {
        self.0.update(index, f)
    }
}
