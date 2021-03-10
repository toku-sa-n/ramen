// SPDX-License-Identifier: GPL-3.0-or-later

pub(super) mod response;

use num_derive::FromPrimitive;

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
    const SIGNATURE: u32 = 0x4342_5355;
}

#[repr(transparent)]
#[derive(Default)]
pub(super) struct CommandDataBlock([u8; 16]);
impl CommandDataBlock {
    pub(super) fn inquiry() -> Self {
        let mut b = Self::default();
        b.set_command(Command::Inquiry);
        b.0[4] = 0x24;
        b
    }

    pub(super) fn read_capacity() -> Self {
        let mut b = Self::default();
        b.set_command(Command::ReadCapacity);
        b
    }

    pub(super) fn read10() -> Self {
        let mut b = Self::default();
        b.set_command(Command::Read10);
        b.0[8] = 0x40;
        b
    }

    pub(super) fn write10() -> Self {
        let mut b = Self::default();
        b.set_command(Command::Write10);
        b.0[8] = 0x40;
        b
    }

    fn set_command(&mut self, c: Command) {
        self.0[0] = c.into();
    }
}

#[repr(u8)]
enum Command {
    Inquiry = 0x12,
    ReadCapacity = 0x25,
    Read10 = 0x28,
    Write10 = 0x2a,
}
impl From<Command> for u8 {
    fn from(c: Command) -> Self {
        c as u8
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
