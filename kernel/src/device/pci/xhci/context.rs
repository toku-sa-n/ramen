// SPDX-License-Identifier: GPL-3.0-or-later

use super::register::Registers;
use crate::mem::allocator::page_box::PageBox;
use alloc::rc::Rc;
use bitfield::bitfield;
use core::cell::RefCell;
use x86_64::PhysAddr;

pub enum Input {
    Bit32(PageBox<InputWithControl32Bit>),
    Bit64(PageBox<InputWithControl64Bit>),
}
impl Input {
    pub fn null(registers: &Rc<RefCell<Registers>>) -> Self {
        if Self::csz(&registers.borrow()) {
            Self::Bit64(PageBox::new(InputWithControl64Bit::null()))
        } else {
            Self::Bit32(PageBox::new(InputWithControl32Bit::null()))
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

    fn csz(registers: &Registers) -> bool {
        let params1 = registers.hc_capability.hc_cp_params_1.read();
        params1.csz()
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
    fn set_aflag(&mut self, inde: usize);
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
}

#[repr(C)]
pub struct Device {
    pub slot: Slot,
    pub ep_0: Endpoint,
    ep_inout: [EndpointOutIn; 15],
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
    out: Endpoint,
    input: Endpoint,
}
impl EndpointOutIn {
    fn null() -> Self {
        Self {
            out: Endpoint::null(),
            input: Endpoint::null(),
        }
    }
}

pub type Endpoint = EndpointStructure<[u32; 8]>;
bitfield! {
    #[repr(transparent)]
    #[derive(Copy,Clone)]
    pub struct EndpointStructure([u32]);
    impl Debug;
    pub u32, _, set_endpoint_type_as_u32: 32+5, 32+3;
    pub u32, _, set_max_packet_size: 32+31, 32+16;
    u64, _, set_dequeue_ptr_as_u64: 96+31, 64;
    pub _, set_dequeue_cycle_state: 64;
    pub u32, _, set_error_count: 32+2, 32+1;
}
impl Endpoint {
    pub fn set_endpoint_type(&mut self, ty: EndpointType) {
        self.set_endpoint_type_as_u32(ty as u32);
    }

    pub fn set_dequeue_ptr(&mut self, addr: PhysAddr) {
        assert!(addr.is_aligned(16_u64));
        self.set_dequeue_ptr_as_u64(addr.as_u64());
    }

    fn null() -> Self {
        Self([0; 8])
    }
}

pub enum EndpointType {
    Control = 4,
}
