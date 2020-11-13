// SPDX-License-Identifier: GPL-3.0-or-later

use {
    bitfield::bitfield,
    core::ops::{Deref, DerefMut},
};

pub struct InputContext {
    pub input_control_context: InputControlContext,
    endpoint_context: [EndpointContext; 32],
}

#[repr(transparent)]
pub struct InputControlContext([u32; 8]);
impl InputControlContext {
    pub fn set_aflag(&mut self, index: usize) {
        assert!(index < 32);
        self.0[1] |= 1 << index;
    }
}

#[repr(transparent)]
pub struct EndpointContext([u32; 8]);

pub struct SlotContext(SlotContextStructure<[u32; 8]>);
impl Deref for SlotContext {
    type Target = SlotContextStructure<[u32; 8]>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for SlotContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
bitfield! {
    #[repr(transparent)]
    pub struct SlotContextStructure([u32]);

    pub u8, _, set_context_entries: 31, 27;
}
