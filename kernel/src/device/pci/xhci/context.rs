// SPDX-License-Identifier: GPL-3.0-or-later

use {
    bitfield::bitfield,
    core::ops::{Deref, DerefMut},
};

pub struct Input {
    pub input_control: InputControl,
    pub device: Device,
}
impl Input {
    pub fn null() -> Self {
        Self {
            input_control: InputControl::null(),
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

pub struct Device {
    slot: Slot,
    pub ep_0: Endpoint,
    ep_inout: [EndpointOutIn; 15],
}
impl Device {
    fn null() -> Self {
        Self {
            slot: Slot::null(),
            ep_0: Endpoint::null(),
            ep_inout: [EndpointOutIn::null(); 15],
        }
    }
}

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

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Endpoint(pub EndpointStructure<[u32; 8]>);
impl Endpoint {
    fn null() -> Self {
        Self(EndpointStructure::null())
    }
}
bitfield! {
    #[repr(transparent)]
    #[derive(Copy,Clone)]
    pub struct EndpointStructure([u32]);
    impl Debug;
    pub u32, _, set_endpoint_type_as_u32: 32+5, 32+3;
    pub u32, _, set_max_packet_size: 32+31, 32+16;
    pub u64, _, set_dequeue_ptr: 96+31, 64;
    pub _, set_dequeue_cycle_state: 64;
    pub u32, _, set_error_count: 32+2, 32+1;
}
impl EndpointStructure<[u32; 8]> {
    pub fn set_endpoint_type(&mut self, ty: EndpointType) {
        self.set_endpoint_type_as_u32(ty as u32);
    }

    fn null() -> Self {
        Self([0; 8])
    }
}

pub enum EndpointType {
    Control = 4,
}

pub struct Slot(SlotStructure<[u32; 8]>);
impl Slot {
    pub fn null() -> Self {
        Self(SlotStructure::null())
    }
}
impl Deref for Slot {
    type Target = SlotStructure<[u32; 8]>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Slot {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
bitfield! {
    #[repr(transparent)]
    pub struct SlotStructure([u32]);

    pub u8, _, set_context_entries: 31, 27;
}
impl<const N: usize> SlotStructure<[u32; N]> {
    fn null() -> Self {
        Self([0; N])
    }
}
