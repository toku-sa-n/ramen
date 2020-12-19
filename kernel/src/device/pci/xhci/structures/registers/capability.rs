// SPDX-License-Identifier: GPL-3.0-or-later

use crate::mem::accessor::Accessor;
use bitfield::bitfield;
use os_units::Bytes;
use x86_64::PhysAddr;

pub struct Capability {
    pub cap_length: Accessor<Len>,
    pub hcs_params_1: Accessor<StructuralParameters1>,
    pub hcs_params_2: Accessor<StructuralParameters2>,
    pub hc_cp_params_1: Accessor<HCCapabilityParameters1>,
    pub db_off: Accessor<DoorbellOffset>,
    pub rts_off: Accessor<RuntimeRegisterSpaceOffset>,
}

impl Capability {
    /// SAFETY: This method is unsafe because if `mmio_base` is not the valid MMIO base address, it
    /// can violate memory safety.
    pub unsafe fn new(mmio_base: PhysAddr) -> Self {
        let cap_length = Accessor::new(mmio_base, Bytes::new(0));
        let hcs_params_1 = Accessor::new(mmio_base, Bytes::new(0x04));
        let hcs_params_2 = Accessor::new(mmio_base, Bytes::new(0x08));
        let hc_cp_params_1 = Accessor::new(mmio_base, Bytes::new(0x10));
        let db_off = Accessor::new(mmio_base, Bytes::new(0x14));
        let rts_off = Accessor::new(mmio_base, Bytes::new(0x18));

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

#[repr(transparent)]
pub struct Len(u8);
impl Len {
    pub fn get(&self) -> usize {
        self.0 as _
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct StructuralParameters1(u32);
    pub u8, max_slots, _: 7, 0;
    pub u8, max_ports, _: 31, 24;
}

bitfield! {
    #[repr(transparent)]
    pub struct StructuralParameters2(u32);
    erst_max, _: 7, 4;
}
impl StructuralParameters2 {
    pub fn powered_erst_max(&self) -> u16 {
        2_u16.pow(self.erst_max())
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct HCCapabilityParameters1(u32);
    pub csz, _: 2;
    pub xhci_extended_capabilities_pointer,_: 31,16;
}

#[repr(transparent)]
pub struct DoorbellOffset(u32);
impl DoorbellOffset {
    pub fn get(&self) -> u32 {
        self.0
    }
}

#[repr(transparent)]
pub struct RuntimeRegisterSpaceOffset(u32);
impl RuntimeRegisterSpaceOffset {
    pub fn get(&self) -> u32 {
        self.0 & !0x1f
    }
}
