// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{raw, CycleBit},
    bitfield::bitfield,
    core::convert::TryFrom,
    os_units::Bytes,
};

#[derive(Debug)]
pub enum Trb {
    Noop(Noop),
    CommandComplete(CommandComplete),
}
impl Trb {
    pub const SIZE: Bytes = Bytes::new(16);
    pub fn new_noop(cycle_bit: CycleBit) -> Self {
        Self::Noop(Noop::new(cycle_bit))
    }
}
impl TryFrom<raw::Trb> for Trb {
    type Error = Error;

    fn try_from(raw: raw::Trb) -> Result<Self, Self::Error> {
        match raw.ty() {
            x if x == Noop::ID => Ok(Self::Noop(Noop::from(raw))),
            x if x == CommandComplete::ID => Ok(Self::CommandComplete(CommandComplete::from(raw))),
            x => {
                warn!("Unrecognized TRB ID: {}", x);
                Err(Error::InvalidId)
            }
        }
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidId,
}

bitfield! {
    #[repr(transparent)]
    pub struct Noop(u128);
    impl Debug;
    _, set_cycle_bit: 96;
    u8, trb_type, set_trb_type: 96+15, 96+10;
}
impl Noop {
    const ID: u8 = 23;
    fn new(cycle_bit: CycleBit) -> Self {
        let mut noop = Noop(0);
        noop.set_cycle_bit(cycle_bit.into());
        noop.set_trb_type(Self::ID);

        noop
    }
}
impl From<raw::Trb> for Noop {
    fn from(raw: raw::Trb) -> Self {
        Self(raw.0)
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct CommandComplete(u128);
    impl Debug;
    completion_code, _: 64+31,64+24;
}
impl CommandComplete {
    const ID: u8 = 33;
}
impl From<raw::Trb> for CommandComplete {
    fn from(raw: raw::Trb) -> Self {
        Self(raw.0)
    }
}
