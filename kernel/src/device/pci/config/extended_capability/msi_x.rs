// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{RegisterIndex, Registers},
    crate::accessor,
    crate::device::pci::config::{bar, type_spec::TypeSpec},
    bitfield::bitfield,
    core::convert::From,
    os_units::{Bytes, Size},
};

#[derive(Debug)]
pub struct CapabilitySpecMsiX<'a> {
    table: accessor::slice::Accessor<'a, Element>,
}

impl<'a> CapabilitySpecMsiX<'a> {
    pub fn new(raw: &Registers, base: RegisterIndex, type_spec: &TypeSpec) -> Self {
        let bir = Bir::parse_raw(raw, base);
        let table_offset = TableOffset::parse_raw(raw, base);

        let base_addr = if let TypeSpec::NonBridge(non_bridge) = type_spec {
            non_bridge.base_addr(bir.get())
        } else {
            todo!()
        };

        let num_elements = TableSize::parse_raw(raw, base);

        Self {
            table: accessor::slice::Accessor::new(
                base_addr,
                table_offset.as_bytes().as_usize(),
                num_elements.as_usize(),
            ),
        }
    }
}

struct Bir(bar::Index);
impl Bir {
    fn parse_raw(raw: &Registers, base: RegisterIndex) -> Self {
        Self(bar::Index::new(raw[base + 4] & 0b111))
    }

    fn get(self) -> bar::Index {
        self.0
    }
}

struct TableOffset(Size<Bytes>);
impl TableOffset {
    fn parse_raw(raw: &Registers, base: RegisterIndex) -> Self {
        Self(Size::new((raw[base + 4] & !0xf) as usize))
    }

    fn as_bytes(self) -> Size<Bytes> {
        self.0
    }
}

struct TableSize(u32);
impl TableSize {
    fn parse_raw(raw: &Registers, base: RegisterIndex) -> Self {
        // Table size is N - 1 encoded.
        // See: https://wiki.osdev.org/PCI#Enabling_MSI-X
        Self(((raw[base] >> 16) & 0x7ff) + 1)
    }

    fn as_usize(self) -> usize {
        self.0 as usize
    }
}

bitfield! {
    #[derive(Debug)]
    struct Element(u128);

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
