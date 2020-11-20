// SPDX-License-Identifier: GPL-3.0-or-later

use bitfield::bitfield;
use x86_64::PhysAddr;

#[repr(C)]
pub struct Input {
    pub control: InputControl,
    pub device: Device,
}
impl Input {
    pub fn null() -> Self {
        Self {
            control: InputControl::null(),
            device: Device::null(),
        }
    }
}

#[repr(transparent)]
pub struct InputControl([u32; 8]);
impl InputControl {
    pub fn null() -> Self {
        Self([0; 8])
    }

    pub fn set_aflag(&mut self, index: usize) {
        assert!(index < 32);
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
impl EndpointStructure<[u32; 8]> {
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

pub type Slot = SlotStructure<[u32; 8]>;
bitfield! {
    #[repr(transparent)]
    pub struct SlotStructure([u32]);

    pub u8, _, set_context_entries: 31, 27;
    pub u8, _, set_root_hub_port_number: 32+23, 32+16;
}
impl<const N: usize> SlotStructure<[u32; N]> {
    fn null() -> Self {
        Self([0; N])
    }
}
