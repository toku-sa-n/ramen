// SPDX-License-Identifier: GPL-3.0-or-later

#[repr(C, packed)]
struct ExtendedCapability {
    capability_id: u8,
    next_ptr: u8,
    capability_specific: CapabilitySpecific,
}

enum CapabilitySpecific {}
