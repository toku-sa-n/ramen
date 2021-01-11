// SPDX-License-Identifier: GPL-3.0-or-later

use super::ring::CycleBit;
use crate::{device::pci::xhci, mem::allocator::page_box::PageBox};
use bit_field::BitField;
use bitfield::bitfield;
use core::convert::{TryFrom, TryInto};
use num_derive::FromPrimitive;
use x86_64::PhysAddr;

pub struct Context {
    pub input: Input,
    pub output_device: PageBox<Device>,
}
impl Context {
    pub fn new() -> Self {
        Self {
            input: Input::null(),
            output_device: PageBox::user(Device::null()),
        }
    }
}
impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

pub enum Input {
    Bit32(PageBox<InputWithControl32Bit>),
    Bit64(PageBox<InputWithControl64Bit>),
}
impl Input {
    pub fn null() -> Self {
        if Self::csz() {
            Self::Bit64(PageBox::user(InputWithControl64Bit::null()))
        } else {
            Self::Bit32(PageBox::user(InputWithControl32Bit::null()))
        }
    }

    pub fn control_mut(&mut self) -> &mut dyn InputControl {
        match self {
            Self::Bit32(b32) => &mut b32.control,
            Self::Bit64(b64) => &mut b64.control,
        }
    }

    pub fn device_mut(&mut self) -> &mut Device {
        match self {
            Self::Bit32(b32) => &mut b32.device,
            Self::Bit64(b64) => &mut b64.device,
        }
    }

    pub fn phys_addr(&self) -> PhysAddr {
        match self {
            Self::Bit32(b32) => b32.phys_addr(),
            Self::Bit64(b64) => b64.phys_addr(),
        }
    }

    fn csz() -> bool {
        xhci::handle_registers(|r| {
            let p1 = r.capability.hc_cp_params_1.read();
            p1.csz()
        })
    }
}

#[repr(C)]
pub struct InputWithControl32Bit {
    control: InputControl32Bit,
    device: Device,
}
impl InputWithControl32Bit {
    fn null() -> Self {
        Self {
            control: InputControl32Bit::null(),
            device: Device::null(),
        }
    }
}

#[repr(C)]
pub struct InputWithControl64Bit {
    control: InputControl64Bit,
    device: Device,
}
impl InputWithControl64Bit {
    fn null() -> Self {
        Self {
            control: InputControl64Bit::null(),
            device: Device::null(),
        }
    }
}

pub trait InputControl {
    fn set_aflag(&mut self, i: usize);
    fn clear_aflag(&mut self, i: usize);
}

#[repr(transparent)]
pub struct InputControl32Bit([u32; 8]);
impl InputControl32Bit {
    fn null() -> Self {
        Self([0; 8])
    }
}
impl InputControl for InputControl32Bit {
    fn set_aflag(&mut self, index: usize) {
        assert!(index < 32);
        self.0[1] |= 1 << index;
    }

    fn clear_aflag(&mut self, i: usize) {
        assert!(i < 32);
        self.0[1].set_bit(i, false);
    }
}

#[repr(transparent)]
pub struct InputControl64Bit([u64; 8]);
impl InputControl64Bit {
    fn null() -> Self {
        Self([0; 8])
    }
}
impl InputControl for InputControl64Bit {
    fn set_aflag(&mut self, index: usize) {
        assert!(index < 64);
        self.0[1] |= 1 << index;
    }

    fn clear_aflag(&mut self, i: usize) {
        assert!(i < 64);
        self.0[1].set_bit(i, false);
    }
}

#[repr(C)]
pub struct Device {
    pub slot: Slot,
    pub ep_0: Endpoint,
    pub ep_inout: [EndpointOutIn; 15],
}
impl Device {
    pub fn null() -> Self {
        Self {
            slot: Slot::null(),
            ep_0: Endpoint::null(),
            ep_inout: [EndpointOutIn::null(); 15],
        }
    }
}

pub type Slot = SlotStructure<[u32; 8]>;
bitfield! {
    #[repr(transparent)]
    pub struct SlotStructure([u32]);

    pub u8, _, set_context_entries: 31, 27;
    pub u8, _, set_root_hub_port_number: 32+23, 32+16;
}
impl Slot {
    fn null() -> Self {
        Self([0; 8])
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct EndpointOutIn {
    pub out: Endpoint,
    pub input: Endpoint,
}
impl EndpointOutIn {
    fn null() -> Self {
        Self {
            out: Endpoint::null(),
            input: Endpoint::null(),
        }
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Endpoint([u32; 8]);
impl Endpoint {
    pub fn set_endpoint_type(&mut self, ty: EndpointType) {
        self.0[1].set_bits(3..=5, ty as _);
    }

    pub fn set_max_burst_size(&mut self, sz: u8) {
        self.0[1].set_bits(8..=15, sz.into());
    }

    pub fn set_interval(&mut self, int: u8) {
        self.0[0].set_bits(16..=23, int.into());
    }

    pub fn set_max_primary_streams(&mut self, s: u8) {
        self.0[0].set_bits(10..=14, s.into());
    }

    pub fn set_mult(&mut self, m: u8) {
        self.0[0].set_bits(8..=9, m.into());
    }

    pub fn set_dequeue_ptr(&mut self, a: PhysAddr) {
        assert!(a.is_aligned(16_u64));
        let l = a.as_u64() & 0xffff_ffff;
        let u = a.as_u64() >> 32;

        self.0[2] = u32::try_from(l).unwrap() | self.0[2].get_bit(0) as u32;
        self.0[3] = u.try_into().unwrap();
    }

    pub fn set_max_packet_size(&mut self, sz: u16) {
        self.0[1].set_bits(16..=31, sz.into());
    }

    pub fn set_dequeue_cycle_state(&mut self, c: CycleBit) {
        self.0[2].set_bit(0, c.into());
    }

    pub fn set_error_count(&mut self, c: u8) {
        self.0[1].set_bits(1..=2, c.into());
    }

    fn null() -> Self {
        Self([0; 8])
    }
}

#[derive(PartialEq, Eq, Debug, FromPrimitive)]
pub enum EndpointType {
    IsochronousOut = 1,
    BulkOut = 2,
    InterruptOut = 3,
    Control = 4,
    IsochronousIn = 5,
    BulkIn = 6,
    InterruptIn = 7,
}
