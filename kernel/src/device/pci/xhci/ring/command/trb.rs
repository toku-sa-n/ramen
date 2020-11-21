// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::{raw, CycleBit};
use bit_field::BitField;
use core::convert::{TryFrom, TryInto};
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

    pub fn new_enable_slot(c: CycleBit) -> Self {
        Self::EnableSlot(EnableSlot::new(c))
    }

    pub fn new_link(a: PhysAddr, c: CycleBit) -> Self {
        Self::Link(Link::new(a, c))
    }

    pub fn new_address_device(c: CycleBit, input_context_addr: PhysAddr, slot_id: u8) -> Self {
        Self::AddressDevice(AddressDevice::new(c, input_context_addr, slot_id))
    }
}
impl From<Trb> for raw::Trb {
    fn from(t: Trb) -> Self {
        match t {
            Trb::Noop(n) => raw::Trb(n.0),
            Trb::Link(l) => raw::Trb(l.0),
            Trb::EnableSlot(e) => raw::Trb(e.0),
            Trb::AddressDevice(a) => raw::Trb(a.0),
        }
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct Link(pub [u32; 4]);
impl Link {
    const ID: u8 = 6;
    fn new(addr_to_ring: PhysAddr, cycle_bit: CycleBit) -> Self {
        assert!(addr_to_ring.is_aligned(u64::try_from(Trb::SIZE.as_usize()).unwrap()));
        let mut trb = Self([0; 4]);
        trb.set_cycle_bit(cycle_bit);
        trb.set_trb_type(Self::ID);
        trb.set_addr(addr_to_ring.as_u64());
        trb
    }

    fn set_addr(&mut self, a: u64) {
        let l = a & 0xffff_ffff;
        let u = a >> 32;
        self.0[0] = l.try_into().unwrap();
        self.0[1] = u.try_into().unwrap();
    }

    fn set_cycle_bit(&mut self, c: CycleBit) {
        self.0[3].set_bit(0, c.into());
    }

    fn set_trb_type(&mut self, t: u8) {
        self.0[3].set_bits(10..=15, t.into());
    }
}
impl From<raw::Trb> for Link {
    fn from(raw: raw::Trb) -> Self {
        Self(raw.0)
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct EnableSlot(pub [u32; 4]);
impl EnableSlot {
    const ID: u8 = 9;
    pub fn new(cycle_bit: CycleBit) -> Self {
        let mut enable_slot = Self([0; 4]);
        enable_slot.set_cycle_bit(cycle_bit);
        enable_slot.set_trb_type(Self::ID);
        enable_slot
    }

    fn set_cycle_bit(&mut self, c: CycleBit) {
        self.0[3].set_bit(0, c.into());
    }

    fn set_trb_type(&mut self, t: u8) {
        self.0[3].set_bits(10..=15, t.into());
    }
}
impl From<raw::Trb> for EnableSlot {
    fn from(raw: raw::Trb) -> Self {
        Self(raw.0)
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct AddressDevice(pub [u32; 4]);
impl AddressDevice {
    const ID: u8 = 11;
    pub fn new(cycle_bit: CycleBit, addr_to_input_context: PhysAddr, slot_id: u8) -> Self {
        let mut trb = Self([0; 4]);

        assert!(addr_to_input_context.is_aligned(16_u64));
        trb.set_input_context_ptr_as_u64(addr_to_input_context.as_u64());
        trb.set_cycle_bit(cycle_bit);
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

    fn set_cycle_bit(&mut self, c: CycleBit) {
        self.0[3].set_bit(0, c.into());
    }

    fn set_trb_type(&mut self, t: u8) {
        self.0[3].set_bits(10..=15, t.into());
    }

    fn set_slot_id(&mut self, id: u8) {
        self.0[3].set_bits(24..=31, id.into());
    }
}
impl From<raw::Trb> for AddressDevice {
    fn from(raw: raw::Trb) -> Self {
        Self(raw.0)
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct Noop(pub [u32; 4]);
impl Noop {
    const ID: u8 = 23;
    fn new(cycle_bit: CycleBit) -> Self {
        let mut noop = Self([0; 4]);
        noop.set_cycle_bit(cycle_bit);
        noop.set_trb_type(Self::ID);

        noop
    }

    fn set_cycle_bit(&mut self, c: CycleBit) {
        self.0[3].set_bit(0, c.into());
    }

    fn set_trb_type(&mut self, t: u8) {
        self.0[3].set_bits(10..=15, t.into());
    }
}
