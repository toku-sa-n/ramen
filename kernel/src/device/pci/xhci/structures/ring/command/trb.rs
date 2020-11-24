// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::{CycleBit, Link};
use crate::add_trb;
use bit_field::BitField;
use core::convert::TryInto;
use os_units::Bytes;
use x86_64::PhysAddr;

pub enum Trb {
    Noop(Noop),
    Link(Link),
    EnableSlot(EnableSlot),
    AddressDevice(AddressDevice),
}
impl Trb {
    pub const SIZE: Bytes = Bytes::new(16);

    pub fn new_enable_slot() -> Self {
        Self::EnableSlot(EnableSlot::new())
    }

    pub fn new_link(a: PhysAddr) -> Self {
        Self::Link(Link::new(a))
    }

    pub fn new_address_device(input_context_addr: PhysAddr, slot_id: u8) -> Self {
        Self::AddressDevice(AddressDevice::new(input_context_addr, slot_id))
    }

    pub fn set_c(&mut self, c: CycleBit) {
        match self {
            Self::Noop(n) => n.set_cycle_bit(c),
            Self::Link(l) => l.set_cycle_bit(c),
            Self::EnableSlot(e) => e.set_cycle_bit(c),
            Self::AddressDevice(a) => a.set_cycle_bit(c),
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
        }
    }
}

add_trb!(EnableSlot);
impl EnableSlot {
    const ID: u8 = 9;
    pub fn new() -> Self {
        let mut enable_slot = Self([0; 4]);
        enable_slot.set_trb_type(Self::ID);
        enable_slot
    }
}
impl Default for EnableSlot {
    fn default() -> Self {
        Self::new()
    }
}

add_trb!(AddressDevice);
impl AddressDevice {
    const ID: u8 = 11;
    pub fn new(addr_to_input_context: PhysAddr, slot_id: u8) -> Self {
        let mut trb = Self([0; 4]);

        assert!(addr_to_input_context.is_aligned(16_u64));
        trb.set_input_context_ptr_as_u64(addr_to_input_context.as_u64());
        trb.set_trb_type(Self::ID);
        trb.set_slot_id(slot_id);
        trb
    }

    fn set_input_context_ptr_as_u64(&mut self, p: u64) {
        let l = p & 0xffff_ffff;
        let u = p >> 32;
        self.0[0] = l.try_into().unwrap();
        self.0[1] = u.try_into().unwrap();
    }

    fn set_slot_id(&mut self, id: u8) {
        self.0[3].set_bits(24..=31, id.into());
    }
}

add_trb!(Noop);
impl Noop {
    const ID: u8 = 23;
    fn new(cycle_bit: CycleBit) -> Self {
        let mut noop = Self([0; 4]);
        noop.set_cycle_bit(cycle_bit);
        noop.set_trb_type(Self::ID);

        noop
    }
}
