// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryFrom;

#[repr(C, packed)]
pub(super) struct CommandBlockWrapper {
    header: CommandBlockWrapperHeader,
    data: CommandDataBlock,
}
impl CommandBlockWrapper {
    pub(super) fn new(header: CommandBlockWrapperHeader, data: CommandDataBlock) -> Self {
        Self { header, data }
    }
}

#[repr(C, packed)]
#[derive(Builder)]
#[builder(no_std)]
pub(super) struct CommandBlockWrapperHeader {
    #[builder(default = "CommandBlockWrapperHeader::SIGNATURE")]
    signature: u32,
    #[builder(default = "0")]
    tag: u32,
    transfer_length: u32,
    flags: u8,
    lun: u8,
    command_len: u8,
}
impl CommandBlockWrapperHeader {
    const SIGNATURE: u32 = 0x43425355;
}

#[repr(transparent)]
#[derive(Default)]
pub(super) struct CommandDataBlock([u8; 16]);
impl CommandDataBlock {
    pub(super) fn inquiry() -> Self {
        let mut b = Self::default();
        b.0[0] = 0x12;
        b.0[4] = 0x24;
        b
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub(super) struct Inquiry([u8; 36]);
impl TryFrom<[u8; 36]> for Inquiry {
    type Error = Invalid;

    fn try_from(value: [u8; 36]) -> Result<Self, Self::Error> {
        let peripheral_device_type = value[0] & 0b1_1111;
        let peripheral_qualifier = value[0] >> 5;
        let response_data_format = value[3] & 0b1111;

        if ![0x00, 0x05].contains(&peripheral_device_type) {
            Err(Invalid::PeripheralDeviceType(peripheral_device_type))
        } else if peripheral_qualifier != 0 {
            Err(Invalid::PeripheralQualifier(peripheral_qualifier))
        } else if ![0x01, 0x02].contains(&response_data_format) {
            Err(Invalid::ResponseDataFormat(response_data_format))
        } else {
            Ok(Self(value))
        }
    }
}

#[derive(Debug)]
pub(super) enum Invalid {
    PeripheralDeviceType(u8),
    PeripheralQualifier(u8),
    ResponseDataFormat(u8),
}
