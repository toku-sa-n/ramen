// SPDX-License-Identifier: GPL-3.0-or-later

pub struct InputContext {
    input_control_context: InputControlContext,
    endpoint_context: [EndpointContext; 32],
}

#[repr(transparent)]
pub struct InputControlContext([u32; 8]);

#[repr(transparent)]
pub struct EndpointContext([u32; 8]);
