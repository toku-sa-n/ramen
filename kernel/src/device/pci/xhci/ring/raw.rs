// SPDX-License-Identifier: GPL-3.0-or-later

use super::{trb, CycleBit};
use crate::mem::allocator::page_box::PageBox;
use bit_field::{BitArray, BitField};
use core::{
    convert::TryInto,
    ops::{Index, IndexMut},
};
use x86_64::PhysAddr;

pub struct Ring(PageBox<[Trb]>);
impl Ring {
    pub fn new(num_trb: usize) -> Self {
        Self(PageBox::new_slice(Trb::null(), num_trb))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn phys_addr(&self) -> PhysAddr {
        self.0.phys_addr()
    }
}
impl Index<usize> for Ring {
    type Output = Trb;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
impl IndexMut<usize> for Ring {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Trb(pub [u32; 4]);
impl Trb {
    pub fn cycle_bit(self) -> CycleBit {
        self.into()
    }

    pub fn ty(self) -> u8 {
        self.0[3].get_bits(10..=15).try_into().unwrap()
    }

    fn null() -> Self {
        Self([0; 4])
    }
}
impl From<trb::Trb> for Trb {
    fn from(trb: trb::Trb) -> Self {
        match trb {
            trb::Trb::Noop(noop) => Self(noop.0),
            trb::Trb::CommandComplete(command_complete) => Self(command_complete.0),
            trb::Trb::Link(link) => Self(link.0),
            trb::Trb::PortStatusChange(change) => Self(change.0),
            trb::Trb::EnableSlot(enable) => Self(enable.0),
            trb::Trb::AddressDevice(address) => Self(address.0),
        }
    }
}
impl From<Trb> for CycleBit {
    fn from(raw: Trb) -> Self {
        Self(raw.0[3].get_bit(0))
    }
}
