// SPDX-License-Identifier: GPL-3.0-or-later

use {
    bitfield::bitfield,
    core::ops::{Deref, DerefMut},
};

pub struct Input {
    pub input_control_context: InputControl,
    endpoint_context: [Endpoint; 32],
}

#[repr(transparent)]
pub struct InputControl([u32; 8]);
impl InputControl {
    pub fn set_aflag(&mut self, index: usize) {
        assert!(index < 32);
        self.0[1] |= 1 << index;
    }
}

#[repr(transparent)]
pub struct Endpoint([u32; 8]);

pub struct Slot(SlotStructure<[u32; 8]>);
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
