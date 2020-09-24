// SPDX-License-Identifier: GPL-3.0-or-later

#[repr(C, packed)]
pub struct ExtendedCapability<T: CapabilitySpecific> {
    capability_id: u8,
    next_ptr: u8,
    capability_specific: T,
}

pub trait CapabilitySpecific {}
