// SPDX-License-Identifier: GPL-3.0-or-later

use crate::add_trb;
use bit_field::BitField;
use core::convert::{TryFrom, TryInto};
use os_units::Bytes;

#[derive(Debug)]
pub enum Trb {
    CommandCompletion(CommandCompletion),
    PortStatusChange(PortStatusChange),
}
impl Trb {
    pub const SIZE: Bytes = Bytes::new(16);
}
impl TryFrom<[u32; 4]> for Trb {
    type Error = Error;

    fn try_from(r: [u32; 4]) -> Result<Self, Self::Error> {
        let id = r[3].get_bits(10..=15).try_into().unwrap();
        match id {
            CommandCompletion::ID => Ok(Self::CommandCompletion(CommandCompletion(r))),
            PortStatusChange::ID => Ok(Self::PortStatusChange(PortStatusChange(r))),
            _ => Err(Error::UnrecognizedId),
        }
    }
}

add_trb!(CommandCompletion);
impl CommandCompletion {
    const ID: u8 = 33;

    pub fn slot_id(&self) -> u8 {
        self.0[3].get_bits(24..=31).try_into().unwrap()
    }

    pub fn trb_addr(&self) -> u64 {
        let l: u64 = self.0[0].into();
        let u: u64 = self.0[1].into();

        u << 32 | l
    }
}

add_trb!(PortStatusChange);
impl PortStatusChange {
    const ID: u8 = 34;
}

add_trb!(TransferEvent);

#[derive(Debug)]
pub enum Error {
    UnrecognizedId,
}
