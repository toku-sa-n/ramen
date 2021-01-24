// SPDX-License-Identifier: GPL-3.0-or-later

use crate::mem::accessor::Single;
use x86_64::PhysAddr;
use xhci::registers::capability::{
    CapabilityParameters1, CapabilityRegistersLength, DoorbellOffset, RuntimeRegisterSpaceOffset,
    StructuralParameters1, StructuralParameters2,
};

pub struct Capability {
    pub cap_length: Single<CapabilityRegistersLength>,
    pub hcs_params_1: Single<StructuralParameters1>,
    pub hcs_params_2: Single<StructuralParameters2>,
    pub hc_cp_params_1: Single<CapabilityParameters1>,
    pub db_off: Single<DoorbellOffset>,
    pub rts_off: Single<RuntimeRegisterSpaceOffset>,
}

impl Capability {
    /// SAFETY: This method is unsafe because if `mmio_base` is not the valid MMIO base address, it
    /// can violate memory safety.
    pub unsafe fn new(mmio_base: PhysAddr) -> Self {
        let cap_length = crate::mem::accessor::user(mmio_base).expect("Address is not aligned.");
        let hcs_params_1 =
            crate::mem::accessor::user(mmio_base + 0x04_usize).expect("Address is not aligned.");
        let hcs_params_2 =
            crate::mem::accessor::user(mmio_base + 0x08_usize).expect("Address is not aligned.");
        let hc_cp_params_1 =
            crate::mem::accessor::user(mmio_base + 0x10_usize).expect("Address is not aligned.");
        let db_off =
            crate::mem::accessor::user(mmio_base + 0x14_usize).expect("Address is not aligned.");
        let rts_off =
            crate::mem::accessor::user(mmio_base + 0x18_usize).expect("Address is not aligned.");

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
