// SPDX-License-Identifier: GPL-3.0-or-later

use {
    bitfield::bitfield,
    core::ops::{Deref, DerefMut},
};

pub struct Input {
    input_control_context: InputControl,
    endpoint_context: [Endpoint; 32],
}
impl Input {
    pub fn null() -> Self {
        Self {
            input_control_context: InputControl::null(),
            endpoint_context: [Endpoint::null(); 32],
        }
    }

    pub fn init(&mut self) {
        self.input_control_context.init();
    }
}

#[repr(transparent)]
pub struct InputControl([u32; 8]);
impl InputControl {
    pub fn null() -> Self {
        Self([0; 8])
    }

    pub fn init(&mut self) {
        self.set_aflag(0);
        self.set_aflag(1);
    }

    fn set_aflag(&mut self, index: usize) {
        assert!(index < 32);
        self.0[1] |= 1 << index;
    }
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Endpoint([u32; 8]);
impl Endpoint {
    fn null() -> Self {
        Self([0; 8])
    }
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
