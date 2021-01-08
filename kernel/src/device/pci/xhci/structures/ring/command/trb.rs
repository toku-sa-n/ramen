// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::{CycleBit, Link};
use crate::{add_trb, impl_default_simply_adds_trb_id};
use bit_field::BitField;
use core::convert::TryInto;
use os_units::Bytes;
use x86_64::PhysAddr;

pub enum Trb {
    Noop(Noop),
    Link(Link),
    EnableSlot(EnableSlot),
    AddressDevice(AddressDevice),
    ConfigureEndpoint(ConfigureEndpoint),
}
impl Trb {
    pub const SIZE: Bytes = Bytes::new(16);

    pub fn set_c(&mut self, c: CycleBit) {
        match self {
            Self::Noop(n) => n.set_cycle_bit(c),
            Self::Link(l) => l.set_cycle_bit(c),
            Self::EnableSlot(e) => e.set_cycle_bit(c),
            Self::AddressDevice(a) => a.set_cycle_bit(c),
            Self::ConfigureEndpoint(e) => e.set_cycle_bit(c),
        }
    }
}
impl From<Trb> for [u32; 4] {
    fn from(t: Trb) -> Self {
        match t {
            Trb::Noop(n) => n.0,
            Trb::Link(l) => l.0,
            Trb::EnableSlot(e) => e.0,
            Trb::AddressDevice(a) => a.0,
            Trb::ConfigureEndpoint(c) => c.0,
        }
    }
}

add_trb!(Noop, 23);
impl_default_simply_adds_trb_id!(Noop);

add_trb!(EnableSlot, 9);
impl_default_simply_adds_trb_id!(EnableSlot);
impl EnableSlot {
    pub fn new() -> Self {
        let mut enable_slot = Self([0; 4]);
        enable_slot.set_trb_type();
        enable_slot
    }
}

add_trb!(AddressDevice, 11);
impl_default_simply_adds_trb_id!(AddressDevice);
impl AddressDevice {
    pub fn set_input_context_ptr(&mut self, p: PhysAddr) -> &mut Self {
        assert!(p.is_aligned(16_u64));

        let p = p.as_u64();
        let l = p & 0xffff_ffff;
        let u = p >> 32;
        self.0[0] = l.try_into().unwrap();
        self.0[1] = u.try_into().unwrap();
        self
    }

    pub fn set_slot_id(&mut self, id: u8) -> &mut Self {
        self.0[3].set_bits(24..=31, id.into());
        self
    }
}

add_trb!(ConfigureEndpoint, 12);
impl_default_simply_adds_trb_id!(ConfigureEndpoint);
impl ConfigureEndpoint {
    pub fn set_context_addr(&mut self, a: PhysAddr) -> &mut Self {
        assert!(a.is_aligned(16_u64));

        let l = a.as_u64() & 0xffff_ffff;
        let u = a.as_u64() >> 32;

        self.0[0] = l.try_into().unwrap();
        self.0[1] = u.try_into().unwrap();
        self
    }

    pub fn set_slot_id(&mut self, id: u8) -> &mut Self {
        self.0[3].set_bits(24..=31, id.into());
        self
    }
}
