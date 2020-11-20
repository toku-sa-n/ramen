// SPDX-License-Identifier: GPL-3.0-or-later

use super::{raw, CycleBit};
use bitfield::bitfield;
use core::convert::TryFrom;
use os_units::Bytes;
use x86_64::PhysAddr;

#[derive(Debug)]
pub enum Trb {
    Noop(Noop),
    CommandComplete(CommandComplete),
    Link(Link),
    PortStatusChange(PortStatusChange),
    EnableSlot(EnableSlot),
    AddressDevice(AddressDevice),
}
impl Trb {
    pub const SIZE: Bytes = Bytes::new(16);
    pub fn new_noop(cycle_bit: CycleBit) -> Self {
        Self::Noop(Noop::new(cycle_bit))
    }

    pub fn new_link(addr_to_ring: PhysAddr, cycle_bit: CycleBit) -> Self {
        Self::Link(Link::new(addr_to_ring, cycle_bit))
    }

    pub fn new_enable_slot(cycle_bit: CycleBit) -> Self {
        Self::EnableSlot(EnableSlot::new(cycle_bit))
    }

    pub fn new_address_device(
        cycle_bit: CycleBit,
        addr_to_input_context: PhysAddr,
        slot_id: u8,
    ) -> Self {
        Self::AddressDevice(AddressDevice::new(
            cycle_bit,
            addr_to_input_context,
            slot_id,
        ))
    }
}
impl TryFrom<raw::Trb> for Trb {
    type Error = Error;

    fn try_from(raw: raw::Trb) -> Result<Self, Self::Error> {
        match raw.ty() {
            x if x == Noop::ID => Ok(Self::Noop(Noop::from(raw))),
            x if x == CommandComplete::ID => Ok(Self::CommandComplete(CommandComplete::from(raw))),
            x if x == Link::ID => Ok(Self::Link(Link::from(raw))),
            x if x == PortStatusChange::ID => {
                Ok(Self::PortStatusChange(PortStatusChange::from(raw)))
            }
            x if x == EnableSlot::ID => Ok(Self::EnableSlot(EnableSlot::from(raw))),
            x if x == AddressDevice::ID => Ok(Self::AddressDevice(AddressDevice::from(raw))),
            x => {
                warn!("Unrecognized TRB ID: {}", x);
                Err(Error::InvalidId)
            }
        }
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidId,
}

bitfield! {
    #[repr(transparent)]
    pub struct Noop(u128);
    impl Debug;
    _, set_cycle_bit: 96;
    u8, trb_type, set_trb_type: 96+15, 96+10;
}
impl Noop {
    const ID: u8 = 23;
    fn new(cycle_bit: CycleBit) -> Self {
        let mut noop = Noop(0);
        noop.set_cycle_bit(cycle_bit.into());
        noop.set_trb_type(Self::ID);

        noop
    }
}
impl From<raw::Trb> for Noop {
    fn from(raw: raw::Trb) -> Self {
        Self(raw.0)
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct CommandComplete(u128);
    impl Debug;
    pub u64, addr_to_command_trb, _: 63, 0;
    completion_code, _: 64+31,64+24;
    pub u8, slot_id, _: 96+31, 96+24;
}
impl CommandComplete {
    const ID: u8 = 33;
}
impl From<raw::Trb> for CommandComplete {
    fn from(raw: raw::Trb) -> Self {
        Self(raw.0)
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct Link(u128);
    impl Debug;
    _, set_addr: 63, 0;
    _, set_cycle_bit: 96;
    u8, _, set_trb_type: 96+15,96+10;
}
impl Link {
    const ID: u8 = 6;
    fn new(addr_to_ring: PhysAddr, cycle_bit: CycleBit) -> Self {
        assert!(addr_to_ring.is_aligned(u64::try_from(Trb::SIZE.as_usize()).unwrap()));
        let mut trb = Link(0);
        trb.set_cycle_bit(cycle_bit.into());
        trb.set_trb_type(Self::ID);
        trb.set_addr(addr_to_ring.as_u64().into());
        trb
    }
}
impl From<raw::Trb> for Link {
    fn from(raw: raw::Trb) -> Self {
        Self(raw.0)
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct PortStatusChange(u128);
    impl Debug;
    port_id, _: 31, 24;
    completion_code, _: 64+31, 64+24;
}
impl PortStatusChange {
    const ID: u8 = 34;
}
impl From<raw::Trb> for PortStatusChange {
    fn from(raw: raw::Trb) -> Self {
        Self(raw.0)
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct EnableSlot(u128);
    impl Debug;
    _, set_cycle_bit: 96;
    u8,_, set_trb_type: 96+15, 96+10;
}
impl EnableSlot {
    const ID: u8 = 9;
    pub fn new(cycle_bit: CycleBit) -> Self {
        let mut enable_slot = Self(0);
        enable_slot.set_cycle_bit(cycle_bit.into());
        enable_slot.set_trb_type(Self::ID);
        enable_slot
    }
}
impl From<raw::Trb> for EnableSlot {
    fn from(raw: raw::Trb) -> Self {
        Self(raw.0)
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct AddressDevice(u128);
    impl Debug;
    u64, _, set_input_context_ptr_as_u64: 63, 0;
    _, set_cycle_bit: 96;
    u8, _, set_trb_type: 96+15, 96+10;
    u8 ,_, set_slot_id: 96+31, 96+24;
}
impl AddressDevice {
    const ID: u8 = 11;
    pub fn new(cycle_bit: CycleBit, addr_to_input_context: PhysAddr, slot_id: u8) -> Self {
        let mut trb = Self(0);

        assert!(addr_to_input_context.is_aligned(16_u64));
        trb.set_input_context_ptr_as_u64(addr_to_input_context.as_u64());
        trb.set_cycle_bit(cycle_bit.into());
        trb.set_trb_type(Self::ID);
        trb.set_slot_id(slot_id);
        trb
    }
}
impl From<raw::Trb> for AddressDevice {
    fn from(raw: raw::Trb) -> Self {
        Self(raw.0)
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct SetupStage(u128);
    impl Debug;
    _, set_bm_request_type: 7, 0;
    _, set_b_request: 15, 8;
    _, set_w_value: 31, 16;
    _, set_w_index: 32+15, 32;
    _, set_w_length: 32+31, 32+16;
    _, set_trb_transfer_length: 64+16, 64;
    _, set_cycle_bit: 96;
    _, set_trb_type: 96+15, 96+10;
    _, set_trt: 96+17, 96+16;
}

bitfield! {
    #[repr(transparent)]
    pub struct DataStage(u128);
    impl Debug;
    u64, _, set_data_buffer_as_u64: 63, 0;
    _, set_trb_transfer_length: 64+16, 64;
    _, set_td_size: 64+21, 64+17;
    _, set_cycle_bit: 96;
    _, set_ioc: 96+5;
    _, set_trb_type: 96+15, 96+10;
    _, set_dir: 96+16;
}

bitfield! {
    #[repr(transparent)]
    pub struct StatusStage(u128);
    impl Debug;
    _, set_cycle_bit: 96;
    _, set_trb_type: 96+15, 96+10;
}

bitfield! {
    #[repr(transparent)]
    pub struct TransferEvent(u128);
    impl Debug;
}
