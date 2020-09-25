// SPDX-License-Identifier: GPL-3.0-or-later

pub struct ExtendedCapability {
    capability_id: u8,
    next_ptr: u8,
    capability_spec: CapabilitySpec,
}

enum CapabilitySpec {}
