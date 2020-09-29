// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{RegisterIndex, Registers},
    crate::device::pci::config::{bar, type_spec::TypeSpec},
    bitfield::bitfield,
    core::convert::From,
    os_units::{Bytes, Size},
};

#[derive(Debug)]
pub struct CapabilitySpec<'a> {
    registers: &'a Registers,
    base: RegisterIndex,
}

impl<'a> CapabilitySpec<'a> {
    pub fn new(registers: &'a Registers, base: RegisterIndex) -> Self {
        Self { registers, base }
    }

    pub fn bir(&self) -> Bir {
        Bir::new(self.registers, self.base)
    }
}

pub struct Bir(bar::Index);
impl Bir {
    fn new(registers: &Registers, base: RegisterIndex) -> Self {
        Self(bar::Index::new(registers.get(base + 4) & 0b111))
    }
}
impl From<Bir> for bar::Index {
    fn from(bir: Bir) -> Self {
        bir.0
    }
}

struct TableOffset(Size<Bytes>);
impl TableOffset {
    fn parse_raw(raw: &Registers, base: RegisterIndex) -> Self {
        Self(Size::new((raw.get(base + 4) & !0xf) as usize))
    }

    fn as_bytes(&self) -> Size<Bytes> {
        self.0
    }
}

struct TableSize(u32);
impl TableSize {
    fn parse_raw(raw: &Registers, base: RegisterIndex) -> Self {
        // Table size is N - 1 encoded.
        // See: https://wiki.osdev.org/PCI#Enabling_MSI-X
        Self(((raw.get(base) >> 16) & 0x7ff) + 1)
    }

    fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

bitfield! {
    #[derive(Debug)]
    pub struct Element(u128);

    u32, from into MessageAddress, message_address,set_message_address: 31, 0;
    u32, from into MessageData, message_data, set_message_data: 95, 64;
    masked, set_mask: 96;
}

bitfield! {
    struct MessageAddress(u32);

    redirection_hint, set_redirection_hint: 3;
    destination_id, set_destination_id: 19, 12;
}

impl From<MessageAddress> for u32 {
    fn from(address: MessageAddress) -> Self {
        address.0
    }
}

impl From<u32> for MessageAddress {
    fn from(val: u32) -> Self {
        Self(val)
    }
}

bitfield! {
    struct MessageData(u32);

    vector, set_vector: 7, 0;
    delivery_mode, set_delivery_mode: 10, 8;
    level, set_level: 14;
    trigger_mode, set_trigger_mode: 15;
}

impl MessageData {
    fn set_level_trigger(&mut self) {
        self.set_trigger_mode(true);
        self.set_level(true);
    }
}

impl From<MessageData> for u32 {
    fn from(address: MessageData) -> Self {
        address.0
    }
}

impl From<u32> for MessageData {
    fn from(val: u32) -> Self {
        Self(val)
    }
}
