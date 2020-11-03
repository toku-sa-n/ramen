// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::hc_capability::DoorbellOffset,
    crate::mem::accessor::Accessor,
    core::{
        convert::TryInto,
        ops::{Index, IndexMut},
    },
    os_units::Bytes,
    x86_64::PhysAddr,
};

const NUM_OF_REGISTERS: usize = 256;

pub struct Array(Accessor<[u32]>);
impl Array {
    pub fn new(mmio_base: PhysAddr, db_off: &DoorbellOffset) -> Self {
        Self(Accessor::new_slice(
            mmio_base,
            Bytes::new(db_off.get().try_into().unwrap()),
            NUM_OF_REGISTERS,
        ))
    }
}
impl Index<usize> for Array {
    type Output = u32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
impl IndexMut<usize> for Array {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}
