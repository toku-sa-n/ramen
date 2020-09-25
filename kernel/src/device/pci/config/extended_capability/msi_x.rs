// SPDX-License-Identifier: GPL-3.0-or-later

use {crate::accessor, bitfield::bitfield, core::convert::From};

pub(super) struct CapabilitySpecMsiX<'a> {
    table: accessor::slice::Accessor<'a, Element>,
}

bitfield! {
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
