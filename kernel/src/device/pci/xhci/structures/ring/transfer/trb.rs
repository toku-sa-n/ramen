// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::{raw, CycleBit};
use crate::add_trb;
use bit_field::BitField;
use bitfield::bitfield;
use core::convert::{TryFrom, TryInto};
use os_units::Bytes;
use x86_64::PhysAddr;

pub enum Trb {
    SetupStageStructure,
}
impl Trb {
    pub const SIZE: Bytes = Bytes::new(16);

    pub fn new_link(a: PhysAddr, c: CycleBit) -> Self {
        unimplemented!()
    }
}
impl TryFrom<raw::Trb> for Trb {
    type Error = Error;

    fn try_from(r: raw::Trb) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}
impl From<Trb> for raw::Trb {
    fn from(t: Trb) -> Self {
        unimplemented!()
    }
}

add_trb!(SetupStage);
impl SetupStage {
    fn set_request_type(&mut self, t: u8) {
        self.0[0].set_bits(0..=7, t.into());
    }

    fn set_request(&mut self, r: u8) {
        self.0[0].set_bits(8..=15, r.into());
    }

    fn set_value(&mut self, v: u16) {
        self.0[0].set_bits(16..=31, v.into());
    }

    fn set_index(&mut self, i: u16) {
        self.0[1].set_bits(0..=15, i.into());
    }

    fn set_length(&mut self, l: u16) {
        self.0[1].set_bits(16..=31, l.into());
    }

    fn set_trb_transfer_length(&mut self, l: u32) {
        self.0[2].set_bits(0..=16, l);
    }

    fn set_trt(&mut self, t: u8) {
        self.0[3].set_bits(16..=17, t.into());
    }
}

add_trb!(DataStage);
impl DataStage {
    fn set_data_buf(&mut self, b: PhysAddr) {
        let l = b.as_u64() & 0xffff_ffff;
        let u = b.as_u64() >> 32;

        self.0[0] = l.try_into().unwrap();
        self.0[1] = u.try_into().unwrap();
    }

    fn set_transfer_length(&mut self, l: u32) {
        self.0[2].set_bits(0..=16, l);
    }

    fn set_td_size(&mut self, s: u8) {
        self.0[2].set_bits(17..=21, s.into());
    }

    fn set_dir(&mut self, d: bool) {
        self.0[3].set_bit(16, d);
    }
}

add_trb!(StatusStage);

pub type TransferEvent = TransferEventStructure<[u32; 4]>;
bitfield! {
    #[repr(transparent)]
    pub struct TransferEventStructure([u32]);
    impl Debug;
}

#[derive(Debug)]
pub enum Error {
    UnrecognizedId,
}
