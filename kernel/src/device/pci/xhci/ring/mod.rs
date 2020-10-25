// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::mem::allocator::page_box::PageBox,
    core::ops::{Index, IndexMut},
    x86_64::PhysAddr,
};

pub mod command;
mod event;
mod trb;

struct Raw(PageBox<[trb::Raw]>);
impl Raw {
    fn new(num_trb: usize) -> Self {
        Self(PageBox::new_slice(num_trb))
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn phys_addr(&self) -> PhysAddr {
        self.0.phys_addr()
    }
}
impl Index<usize> for Raw {
    type Output = trb::Raw;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
impl IndexMut<usize> for Raw {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq)]
struct CycleBit(bool);
impl CycleBit {
    fn new(val: bool) -> Self {
        Self(val)
    }

    fn toggle(&mut self) {
        self.0 = !self.0;
    }
}
impl From<CycleBit> for bool {
    fn from(cycle_bit: CycleBit) -> Self {
        cycle_bit.0
    }
}
