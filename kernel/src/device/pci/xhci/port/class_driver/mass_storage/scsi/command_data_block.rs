// SPDX-License-Identifier: GPL-3.0-or-later

use byteorder::{BigEndian, ByteOrder};

#[derive(Copy, Clone)]
pub(in super::super) enum CommandDataBlock {
    Inquiry(Inquiry),
    ReadCapacity(ReadCapacity),
    Read10(Read10),
    Write10(Write10),
}
impl From<CommandDataBlock> for [u8; 16] {
    fn from(cdb: CommandDataBlock) -> Self {
        match cdb {
            CommandDataBlock::Inquiry(i) => i.0,
            CommandDataBlock::ReadCapacity(r) => r.0,
            CommandDataBlock::Read10(r) => r.0,
            CommandDataBlock::Write10(w) => w.0,
        }
    }
}

#[derive(Copy, Clone)]
pub(in super::super) struct Inquiry([u8; 16]);
impl Inquiry {
    fn set_command(&mut self, c: Command) -> &mut Self {
        self.0[0] = c.into();
        self
    }

    fn set_length(&mut self, l: u16) -> &mut Self {
        BigEndian::write_u16(&mut self.0[3..=4], l);
        self
    }
}
impl Default for Inquiry {
    fn default() -> Self {
        *Self([0; 16]).set_command(Command::Inquiry).set_length(0x24)
    }
}
impl From<Inquiry> for CommandDataBlock {
    fn from(i: Inquiry) -> Self {
        CommandDataBlock::Inquiry(i)
    }
}

#[derive(Copy, Clone)]
pub(in super::super) struct ReadCapacity([u8; 16]);
impl ReadCapacity {
    fn set_command(&mut self, c: Command) -> &mut Self {
        self.0[0] = c.into();
        self
    }
}
impl Default for ReadCapacity {
    fn default() -> Self {
        *Self([0; 16]).set_command(Command::ReadCapacity)
    }
}
impl From<ReadCapacity> for CommandDataBlock {
    fn from(r: ReadCapacity) -> Self {
        CommandDataBlock::ReadCapacity(r)
    }
}

#[derive(Copy, Clone)]
pub(in super::super) struct Read10([u8; 16]);
impl Read10 {
    pub(in super::super) fn new(lba: u32, num_of_blocks: u16) -> Self {
        *Self([0; 16])
            .set_command(Command::Read10)
            .set_lba(lba)
            .set_num_of_blocks(num_of_blocks)
    }

    fn set_command(&mut self, c: Command) -> &mut Self {
        self.0[0] = c.into();
        self
    }

    fn set_lba(&mut self, l: u32) -> &mut Self {
        BigEndian::write_u32(&mut self.0[2..6], l);
        self
    }

    fn set_num_of_blocks(&mut self, n: u16) -> &mut Self {
        BigEndian::write_u16(&mut self.0[7..=8], n);
        self
    }
}
impl From<Read10> for CommandDataBlock {
    fn from(r: Read10) -> Self {
        CommandDataBlock::Read10(r)
    }
}

#[derive(Copy, Clone)]
pub(in super::super) struct Write10([u8; 16]);
impl Write10 {
    pub(in super::super) fn new(lba: u32, num_of_blocks: u16) -> Self {
        *Self([0; 16])
            .set_command(Command::Write10)
            .set_lba(lba)
            .set_num_of_blocks(num_of_blocks)
    }

    fn set_command(&mut self, c: Command) -> &mut Self {
        self.0[0] = c.into();
        self
    }

    fn set_lba(&mut self, l: u32) -> &mut Self {
        BigEndian::write_u32(&mut self.0[2..6], l);
        self
    }

    fn set_num_of_blocks(&mut self, n: u16) -> &mut Self {
        BigEndian::write_u16(&mut self.0[7..=8], n);
        self
    }
}
impl From<Write10> for CommandDataBlock {
    fn from(w: Write10) -> Self {
        CommandDataBlock::Write10(w)
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
