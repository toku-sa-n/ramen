// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::{TryFrom, TryInto};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

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

#[repr(C, packed)]
pub(super) struct CommandStatus<T>
where
    T: Copy,
{
    status: T,
    wrapper: CommandStatusWrapper,
}
impl<T> CommandStatus<T>
where
    T: Copy,
{
    pub(super) fn status(self) -> T {
        self.status
    }

    pub(super) fn wrapper(self) -> CommandStatusWrapper {
        self.wrapper
    }

    // Because of the blanket implementation, `impl<T,U> TryFrom<CommandStatus<U>> for
    // CommandStatus<T> where T:TryFrom<U>` cannot be implemented.
    pub(super) fn try_into<U>(self) -> Result<CommandStatus<U>, <U as TryFrom<T>>::Error>
    where
        U: TryFrom<T> + Copy,
    {
        Ok(CommandStatus {
            status: U::try_from(self.status)?,
            wrapper: self.wrapper,
        })
    }
}
impl<T> Default for CommandStatus<T>
where
    T: Copy + Default,
{
    fn default() -> Self {
        Self {
            status: T::default(),
            wrapper: CommandStatusWrapper::default(),
        }
    }
}
impl<T> Clone for CommandStatus<T>
where
    T: Copy,
{
    fn clone(&self) -> Self {
        CommandStatus {
            status: self.status,
            wrapper: self.wrapper,
        }
    }
}
impl<T> Copy for CommandStatus<T> where T: Copy {}

#[repr(C, packed)]
#[derive(Copy, Clone, Default)]
pub(super) struct CommandStatusWrapper {
    signature: u32,
    tag: u32,
    data_residue: u32,
    status: u8,
}
impl CommandStatusWrapper {
    pub(super) fn status(&self) -> Result<Status, Invalid> {
        FromPrimitive::from_u8(self.status).ok_or(Invalid::Status(self.status))
    }
}

#[derive(Copy, Clone, Debug, FromPrimitive)]
pub(super) enum Status {
    Good = 0,
}
impl Default for Status {
    fn default() -> Self {
        Self::Good
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub(super) struct Inquiry([u8; 36]);
impl TryFrom<RawInquiry> for Inquiry {
    type Error = Invalid;

    fn try_from(value: RawInquiry) -> Result<Self, Self::Error> {
        let value = value.0;
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

#[repr(transparent)]
#[derive(Copy, Clone)]
pub(super) struct RawInquiry([u8; 36]);
impl Default for RawInquiry {
    fn default() -> Self {
        Self([0; 36])
    }
}

#[derive(Debug)]
pub(super) enum Invalid {
    PeripheralDeviceType(u8),
    PeripheralQualifier(u8),
    ResponseDataFormat(u8),
    Status(u8),
}
