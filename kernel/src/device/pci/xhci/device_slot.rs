// SPDX-License-Identifier: GPL-3.0-or-later

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
