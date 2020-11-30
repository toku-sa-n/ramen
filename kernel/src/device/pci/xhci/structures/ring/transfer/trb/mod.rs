// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::{CycleBit, Link};
use crate::{add_trb, mem::allocator::page_box::PageBox};
use bit_field::BitField;
use control::{Control, DescTyIdx};
use core::convert::TryInto;
use os_units::Bytes;
use x86_64::PhysAddr;

pub mod control;

#[derive(Debug, Copy, Clone)]
pub enum Trb {
    Control(Control),
    Link(Link),
    Normal(Normal),
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

    pub fn new_normal<T: ?Sized>(b: &PageBox<T>) -> Self {
        Self::Normal(Normal::new(b))
    }

    pub fn set_c(&mut self, c: CycleBit) {
        match self {
            Self::Control(co) => co.set_cycle_bit(c),
            Self::Link(l) => l.set_cycle_bit(c),
            Self::Normal(n) => n.set_cycle_bit(c),
        }
    }

    pub fn ioc(&self) -> bool {
        match self {
            Self::Control(c) => c.ioc(),
            Self::Link(_) => false,
            Self::Normal(n) => n.ioc(),
        }
    }
}
impl From<Trb> for [u32; 4] {
    fn from(t: Trb) -> Self {
        match t {
            Trb::Control(c) => c.into(),
            Trb::Link(l) => l.0,
            Trb::Normal(n) => n.0,
        }
    }
}

add_trb!(Normal);
impl Normal {
    const ID: u8 = 1;

    pub fn new<T: ?Sized>(b: &PageBox<T>) -> Self {
        let mut t = Self([0; 4]);
        t.set_buf_ptr(b.phys_addr());
        t.set_transfer_length(b.bytes());
        t.set_ioc(true);
        t.set_trb_type(Self::ID);
        t
    }

    fn set_buf_ptr(&mut self, p: PhysAddr) {
        let l = p.as_u64() & 0xffff_ffff;
        let u = p.as_u64() >> 32;

        self.0[0] = l.try_into().unwrap();
        self.0[1] = u.try_into().unwrap();
    }

    fn set_transfer_length(&mut self, bytes: Bytes) {
        self.0[2].set_bits(0..=16, bytes.as_usize().try_into().unwrap());
    }

    fn set_ioc(&mut self, ioc: bool) {
        self.0[3].set_bit(5, ioc);
    }

    fn ioc(&self) -> bool {
        self.0[3].get_bit(5)
    }
}
