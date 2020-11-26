// SPDX-License-Identifier: GPL-3.0-or-later

use crate::add_trb;
use bit_field::BitField;
use core::convert::{TryFrom, TryInto};
use x86_64::PhysAddr;

#[derive(Debug)]
pub enum Completion {
    Command(CommandCompletion),
    Transfer(TransferEvent),
}
impl Completion {
    pub fn slot_id(&self) -> u8 {
        match self {
            Self::Command(c) => c.slot_id(),
            Self::Transfer(_) => unimplemented!(),
        }
    }

    pub fn addr(&self) -> PhysAddr {
        match self {
            Self::Command(c) => c.trb_addr(),
            Self::Transfer(t) => t.trb_addr(),
        }
    }

    pub fn transfer_length(&self) -> Option<u32> {
        match self {
            Self::Command(_) => None,
            Self::Transfer(t) => Some(t.transfer_length()),
        }
    }
}
impl TryFrom<[u32; 4]> for Completion {
    type Error = super::Error;

    fn try_from(r: [u32; 4]) -> Result<Self, Self::Error> {
        let id: u8 = r[3].get_bits(10..=15).try_into().unwrap();
        match id {
            CommandCompletion::ID => Ok(Self::Command(CommandCompletion(r))),
            TransferEvent::ID => Ok(Self::Transfer(TransferEvent(r))),
            _ => Err(super::Error::UnrecognizedId),
        }
    }
}

add_trb!(CommandCompletion);
impl CommandCompletion {
    const ID: u8 = 33;

    pub fn slot_id(&self) -> u8 {
        self.0[3].get_bits(24..=31).try_into().unwrap()
    }

    pub fn trb_addr(&self) -> PhysAddr {
        let l: u64 = self.0[0].into();
        let u: u64 = self.0[1].into();

        PhysAddr::new(u << 32 | l)
    }
}

add_trb!(TransferEvent);
impl TransferEvent {
    const ID: u8 = 32;

    fn trb_addr(&self) -> PhysAddr {
        let l: u64 = self.0[0].into();
        let u: u64 = self.0[1].into();

        PhysAddr::new(u << 32 | l)
    }

    fn transfer_length(&self) -> u32 {
        self.0[2].get_bits(0..=23)
    }
}
