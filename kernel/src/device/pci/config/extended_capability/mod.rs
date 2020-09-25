// SPDX-License-Identifier: GPL-3.0-or-later

mod msi_x;

use {alloc::vec::Vec, msi_x::CapabilitySpecMsiX};

struct ExtendedCapabilities(Vec<ExtendedCapability>);

pub struct ExtendedCapability {
    capability_id: u8,
    next_ptr: u8,
    capability_spec: CapabilitySpec,
}

enum CapabilitySpec {
    MsiX(CapabilitySpecMsiX),
}
