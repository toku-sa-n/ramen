// SPDX-License-Identifier: GPL-3.0-or-later

use crate::add_trb;
use bit_field::BitField;
use completion::Completion;
use core::convert::{TryFrom, TryInto};
use os_units::Bytes;

pub mod completion;

#[derive(Debug)]
pub enum Trb {
    Completion(Completion),
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
            PortStatusChange::ID => Ok(Self::PortStatusChange(PortStatusChange(r))),
            _ => Ok(Self::Completion(Completion::try_from(r)?)),
        }
    }
}

add_trb!(PortStatusChange, 34);
impl PortStatusChange {
    pub fn port(&self) -> u8 {
        self.0[0].get_bits(24..=31).try_into().unwrap()
    }
}

#[derive(Debug)]
pub enum Error {
    UnrecognizedId,
}
