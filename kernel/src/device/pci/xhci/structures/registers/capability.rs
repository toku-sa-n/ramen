// SPDX-License-Identifier: GPL-3.0-or-later

use crate::mem::accessor::Accessor;
use os_units::Bytes;
use x86_64::PhysAddr;
use xhci::registers::capability::{
    CapabilityParameters1, CapabilityRegistersLength, DoorbellOffset, RuntimeRegisterSpaceOffset,
    StructuralParameters1, StructuralParameters2,
};

pub struct Capability {
    pub cap_length: Accessor<CapabilityRegistersLength>,
    pub hcs_params_1: Accessor<StructuralParameters1>,
    pub hcs_params_2: Accessor<StructuralParameters2>,
    pub hc_cp_params_1: Accessor<CapabilityParameters1>,
    pub db_off: Accessor<DoorbellOffset>,
    pub rts_off: Accessor<RuntimeRegisterSpaceOffset>,
}

impl Capability {
    /// SAFETY: This method is unsafe because if `mmio_base` is not the valid MMIO base address, it
    /// can violate memory safety.
    pub unsafe fn new(mmio_base: PhysAddr) -> Self {
        let cap_length = Accessor::user(mmio_base, Bytes::new(0));
        let hcs_params_1 = Accessor::user(mmio_base, Bytes::new(0x04));
        let hcs_params_2 = Accessor::user(mmio_base, Bytes::new(0x08));
        let hc_cp_params_1 = Accessor::user(mmio_base, Bytes::new(0x10));
        let db_off = Accessor::user(mmio_base, Bytes::new(0x14));
        let rts_off = Accessor::user(mmio_base, Bytes::new(0x18));

        Self {
            cap_length,
            hcs_params_1,
            hcs_params_2,
            hc_cp_params_1,
            db_off,
            rts_off,
        }
    }
}
