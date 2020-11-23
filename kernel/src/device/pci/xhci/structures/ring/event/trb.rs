// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::raw;
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
impl TryFrom<raw::Trb> for Trb {
    type Error = Error;

    fn try_from(raw: raw::Trb) -> Result<Self, Self::Error> {
        if let Ok(c) = CommandCompletion::try_from(raw) {
            Ok(Self::CommandCompletion(c))
        } else {
            Ok(Self::PortStatusChange(PortStatusChange::try_from(raw)?))
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
impl TryFrom<raw::Trb> for CommandCompletion {
    type Error = Error;

    fn try_from(raw: raw::Trb) -> Result<Self, Self::Error> {
        if raw.id() == Self::ID {
            Ok(Self(raw.0))
        } else {
            Err(Error::UnrecognizedId)
        }
    }
}

add_trb!(PortStatusChange);
impl PortStatusChange {
    const ID: u8 = 34;
}
impl TryFrom<raw::Trb> for PortStatusChange {
    type Error = Error;

    fn try_from(raw: raw::Trb) -> Result<Self, Self::Error> {
        if raw.id() == Self::ID {
            Ok(Self(raw.0))
        } else {
            Err(Error::UnrecognizedId)
        }
    }
}

add_trb!(TransferEvent);

#[derive(Debug)]
pub enum Error {
    UnrecognizedId,
}
