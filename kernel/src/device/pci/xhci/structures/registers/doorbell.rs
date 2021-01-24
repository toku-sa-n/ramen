// SPDX-License-Identifier: GPL-3.0-or-later

use crate::mem::accessor::Array as ArrayAccessor;
use core::convert::TryFrom;
use x86_64::PhysAddr;

const NUM_OF_REGISTERS: usize = 256;

pub struct Array(ArrayAccessor<u32>);
impl Array {
    /// Safety: `mmio_base` must be a valid address to the top of the doorbell registers.
    pub unsafe fn new(mmio_base: PhysAddr, db_off: u32) -> Self {
        Self(
            crate::mem::accessor::user_array(
                mmio_base + usize::try_from(db_off).unwrap(),
                NUM_OF_REGISTERS,
            )
            .expect("Address is not aligned"),
        )
    }

    pub fn update<T>(&mut self, index: usize, f: T)
    where
        T: Fn(&mut u32),
    {
        self.0.update_at(index, f)
    }
}
