// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::{raw, CycleBit};
use crate::mem::allocator::page_box::PageBox;
use control::{Control, DescTyIdx};
use core::convert::TryFrom;
use os_units::Bytes;
use x86_64::PhysAddr;

mod control;

pub enum Trb {
    Control(Control),
}
impl Trb {
    pub const SIZE: Bytes = Bytes::new(16);

    pub fn new_get_descriptor<T>(
        b: &PageBox<T>,
        dti: DescTyIdx,
        c: CycleBit,
    ) -> (Self, Self, Self) {
        let (setup, data, status) = Control::new_get_descriptor(b, dti, c);
        (
            Self::Control(setup),
            Self::Control(data),
            Self::Control(status),
        )
    }

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

#[derive(Debug)]
pub enum Error {
    UnrecognizedId,
}
