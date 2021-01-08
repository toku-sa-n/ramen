// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    add_trb,
    device::pci::xhci::structures::{descriptor, ring::CycleBit},
    mem::allocator::page_box::PageBox,
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
    pub fn new_get_descriptor<T: ?Sized>(b: &PageBox<T>, dti: DescTyIdx) -> (Self, Self, Self) {
        let setup = SetupStage::new_get_descriptor(b, dti);
        let data = DataStage::new(b, Direction::In);
        let status = StatusStage::new();

        (Self::Setup(setup), Self::Data(data), Self::Status(status))
    }

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
    fn new_get_descriptor<T: ?Sized>(b: &PageBox<T>, dti: DescTyIdx) -> Self {
        let mut t = Self::null();
        t.set_request_type(0b1000_0000);
        t.set_request(Request::GetDescriptor);
        t.set_value(dti.bits());
        t.set_length(b.bytes().as_usize().try_into().unwrap());
        t.set_trb_transfer_length(8);
        t.set_trb_type();
        t.set_trt(3);
        t
    }

    fn null() -> Self {
        let mut t = Self([0; 4]);
        t.set_idt();
        t
    }

    fn set_idt(&mut self) {
        self.0[3].set_bit(6, true);
    }

    fn set_request_type(&mut self, t: u8) {
        self.0[0].set_bits(0..=7, t.into());
    }

    fn set_request(&mut self, r: Request) {
        self.0[0].set_bits(8..=15, r as _);
    }

    fn set_value(&mut self, v: u16) {
        self.0[0].set_bits(16..=31, v.into());
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

pub struct DescTyIdx {
    ty: descriptor::Ty,
    i: u8,
}
impl DescTyIdx {
    pub fn new(ty: descriptor::Ty, i: u8) -> Self {
        Self { ty, i }
    }
    fn bits(self) -> u16 {
        (self.ty as u16) << 8 | u16::from(self.i)
    }
}

enum Request {
    GetDescriptor = 6,
}

add_trb!(DataStage, 3);
impl DataStage {
    fn new<T: ?Sized>(b: &PageBox<T>, d: Direction) -> Self {
        let mut t = Self::null();
        t.set_data_buf(b.phys_addr());
        t.set_transfer_length(b.bytes().as_usize().try_into().unwrap());
        t.set_trb_type();
        t.set_dir(d);
        t
    }

    fn null() -> Self {
        Self([0; 4])
    }

    fn set_data_buf(&mut self, b: PhysAddr) {
        let l = b.as_u64() & 0xffff_ffff;
        let u = b.as_u64() >> 32;

        self.0[0] = l.try_into().unwrap();
        self.0[1] = u.try_into().unwrap();
    }

    fn set_transfer_length(&mut self, l: u32) {
        self.0[2].set_bits(0..=16, l);
    }

    fn set_dir(&mut self, d: Direction) {
        self.0[3].set_bit(16, d.into());
    }
}

enum Direction {
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
impl StatusStage {
    fn new() -> Self {
        let mut t = Self::null();
        t.set_ioc(true);
        t.set_trb_type();
        t
    }

    fn null() -> Self {
        Self([0; 4])
    }

    fn set_ioc(&mut self, ioc: bool) {
        self.0[3].set_bit(5, ioc);
    }

    fn ioc(&self) -> bool {
        self.0[3].get_bit(5)
    }
}
