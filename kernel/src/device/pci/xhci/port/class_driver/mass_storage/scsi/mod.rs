// SPDX-License-Identifier: GPL-3.0-or-later

mod channel;
pub(super) mod response;

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
    #[builder(default = "334")]
    tag: u32,
    transfer_length: u32,
    flags: u8,
    lun: u8,
    command_len: u8,
}
impl CommandBlockWrapperHeader {
    const SIGNATURE: u32 = 0x4342_5355;
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

    pub(super) fn read_format_capacities() -> Self {
        let mut b = Self::default();
        b.0[0] = 0x23;
        b.0[8] = 0xfc;
        b
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Default)]
pub(super) struct CommandStatusWrapper {
    signature: u32,
    tag: u32,
    data_residue: u32,
    status: u8,
}
impl CommandStatusWrapper {
    pub(super) fn check_corruption(&self) {
        const USBS: u32 = 0x5342_5355;
        let signature = self.signature;

        assert_eq!(
            signature, USBS,
            "The signature of the Command Status Wrapper is wrong."
        );

        info!("Tag: {}", self.tag);

        if let Err(Invalid::Status(s)) = self.status() {
            panic!("Error code {:?}", s);
        }
    }

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

#[derive(Debug)]
pub(super) enum Invalid {
    PeripheralDeviceType(u8),
    PeripheralQualifier(u8),
    ResponseDataFormat(u8),
    Status(u8),
}
