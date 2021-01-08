// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    add_trb,
    device::pci::xhci::structures::{descriptor, ring::CycleBit},
    impl_default_simply_adds_trb_id,
};
use bit_field::BitField;
use core::convert::TryInto;
use x86_64::PhysAddr;

#[derive(Copy, Clone, Debug)]
pub enum Control {
    Setup(SetupStage),
    Data(DataStage),
    Status(StatusStage),
}
impl Control {
    pub fn ioc(&self) -> bool {
        // For the control TRBs, only the Status Stage TRB should handle IOC bit. Refer to the note
        // of xHCI dev manual 6.4.1.2
        if let Self::Status(s) = self {
            s.ioc()
        } else {
            false
        }
    }

    pub fn set_cycle_bit(&mut self, c: CycleBit) {
        match self {
            Self::Setup(s) => s.set_cycle_bit(c),
            Self::Data(d) => d.set_cycle_bit(c),
            Self::Status(s) => s.set_cycle_bit(c),
        }
    }
}
impl From<Control> for [u32; 4] {
    fn from(c: Control) -> Self {
        match c {
            Control::Setup(s) => s.0,
            Control::Data(d) => d.0,
            Control::Status(s) => s.0,
        }
    }
}

add_trb!(SetupStage, 2);
impl SetupStage {
    pub fn set_request_type(&mut self, t: u8) -> &mut Self {
        self.0[0].set_bits(0..=7, t.into());
        self
    }

    pub fn set_request(&mut self, r: Request) -> &mut Self {
        self.0[0].set_bits(8..=15, r as _);
        self
    }

    pub fn set_value(&mut self, v: u16) -> &mut Self {
        self.0[0].set_bits(16..=31, v.into());
        self
    }

    pub fn set_length(&mut self, l: u16) -> &mut Self {
        self.0[1].set_bits(16..=31, l.into());
        self
    }

    pub fn set_trb_transfer_length(&mut self, l: u32) -> &mut Self {
        self.0[2].set_bits(0..=16, l);
        self
    }

    pub fn set_trt(&mut self, t: u8) -> &mut Self {
        self.0[3].set_bits(16..=17, t.into());
        self
    }

    fn set_idt(&mut self) -> &mut Self {
        self.0[3].set_bit(6, true);
        self
    }
}
impl Default for SetupStage {
    fn default() -> Self {
        let mut t = Self([0; 4]);
        t.set_trb_type();
        t.set_idt();
        t
    }
}

pub struct DescTyIdx {
    ty: descriptor::Ty,
    i: u8,
}
impl DescTyIdx {
    pub fn new(ty: descriptor::Ty, i: u8) -> Self {
        Self { ty, i }
    }
    pub fn bits(self) -> u16 {
        (self.ty as u16) << 8 | u16::from(self.i)
    }
}

pub enum Request {
    GetDescriptor = 6,
}

add_trb!(DataStage, 3);
impl_default_simply_adds_trb_id!(DataStage);
impl DataStage {
    pub fn set_data_buf(&mut self, b: PhysAddr) -> &mut Self {
        let l = b.as_u64() & 0xffff_ffff;
        let u = b.as_u64() >> 32;

        self.0[0] = l.try_into().unwrap();
        self.0[1] = u.try_into().unwrap();
        self
    }

    pub fn set_transfer_length(&mut self, l: u32) -> &mut Self {
        self.0[2].set_bits(0..=16, l);
        self
    }

    pub fn set_dir(&mut self, d: Direction) -> &mut Self {
        self.0[3].set_bit(16, d.into());
        self
    }
}

pub enum Direction {
    _Out = 0,
    In = 1,
}
impl From<Direction> for bool {
    fn from(d: Direction) -> Self {
        match d {
            Direction::_Out => false,
            Direction::In => true,
        }
    }
}

add_trb!(StatusStage, 4);
impl_default_simply_adds_trb_id!(StatusStage);
impl StatusStage {
    pub fn set_ioc(&mut self, ioc: bool) -> &mut Self {
        self.0[3].set_bit(5, ioc);
        self
    }

    fn ioc(&self) -> bool {
        self.0[3].get_bit(5)
    }
}
