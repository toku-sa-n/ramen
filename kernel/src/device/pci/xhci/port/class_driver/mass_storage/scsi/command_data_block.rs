// SPDX-License-Identifier: GPL-3.0-or-later

#[repr(transparent)]
#[derive(Default)]
pub(in super::super) struct CommandDataBlock([u8; 16]);
impl CommandDataBlock {
    pub(in super::super) fn inquiry() -> Self {
        let mut b = Self::default();
        b.set_command(Command::Inquiry);
        b.0[4] = 0x24;
        b
    }

    pub(in super::super) fn read_capacity() -> Self {
        let mut b = Self::default();
        b.set_command(Command::ReadCapacity);
        b
    }

    pub(in super::super) fn read10() -> Self {
        let mut b = Self::default();
        b.set_command(Command::Read10);
        b.0[8] = 0x40;
        b
    }

    pub(in super::super) fn write10() -> Self {
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
