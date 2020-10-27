// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{raw, CycleBit},
    bitfield::bitfield,
    core::convert::TryFrom,
};

enum Ty {
    Noop = 8,
}
impl TryFrom<raw::Trb> for Ty {
    type Error = Error;

    fn try_from(raw: raw::Trb) -> Result<Self, Self::Error> {
        let error_num = (raw.0 >> 106) & 0x3f;

        match error_num {
            x if x == Self::Noop as _ => Ok(Self::Noop),
            _ => Err(Error::InvalidId),
        }
    }
}

#[derive(Debug)]
pub enum Trb {
    Noop(Noop),
}
impl Trb {
    pub fn new_noop(cycle_bit: CycleBit) -> Self {
        Self::Noop(Noop::new(cycle_bit))
    }
}
impl TryFrom<raw::Trb> for Trb {
    type Error = Error;

    fn try_from(raw: raw::Trb) -> Result<Self, Self::Error> {
        match Ty::try_from(raw) {
            Ok(ty) => match ty {
                Ty::Noop => Ok(Self::Noop(Noop::from(raw))),
            },
            Err(_) => Err(Error::InvalidId),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidId,
}

bitfield! {
    #[repr(transparent)]
    #[derive(Debug)]
    pub struct Noop(u128);

    _, set_cycle_bit: 96;
    trb_type, set_trb_type: (96+15), (96+10);
}
impl Noop {
    fn new(cycle_bit: CycleBit) -> Self {
        let mut noop = Noop(0);
        noop.set_cycle_bit(cycle_bit.into());
        noop.set_trb_type(Ty::Noop as _);

        noop
    }
}
impl From<raw::Trb> for Noop {
    fn from(raw: raw::Trb) -> Self {
        Self(raw.0)
    }
}
