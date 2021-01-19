// SPDX-License-Identifier: GPL-3.0-or-later

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
