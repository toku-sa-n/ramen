// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::{CycleBit, Link};
use crate::mem::allocator::page_box::PageBox;
use control::{Control, DescTyIdx};
use os_units::Bytes;
use x86_64::PhysAddr;

pub mod control;

#[derive(Copy, Clone)]
pub enum Trb {
    Control(Control),
    Link(Link),
}
impl Trb {
    pub const SIZE: Bytes = Bytes::new(16);

    pub fn new_get_descriptor<T: ?Sized>(b: &PageBox<T>, dti: DescTyIdx) -> (Self, Self, Self) {
        let (setup, data, status) = Control::new_get_descriptor(b, dti);
        (
            Self::Control(setup),
            Self::Control(data),
            Self::Control(status),
        )
    }

    pub fn new_link(a: PhysAddr) -> Self {
        Self::Link(Link::new(a))
    }

    pub fn set_c(&mut self, c: CycleBit) {
        match self {
            Self::Control(co) => co.set_cycle_bit(c),
            Self::Link(l) => l.set_cycle_bit(c),
        }
    }

    pub fn ioc(&self) -> bool {
        match self {
            Self::Control(c) => c.ioc(),
            Self::Link(_) => false,
        }
    }
}
impl From<Trb> for [u32; 4] {
    fn from(t: Trb) -> Self {
        match t {
            Trb::Control(c) => c.into(),
            Trb::Link(l) => l.0,
        }
    }
}
