// SPDX-License-Identifier: GPL-3.0-or-later

use {crate::accessor::single_object::Accessor, bitfield::bitfield, x86_64::PhysAddr};

pub struct HCCapabilityRegisters<'a> {
    pub cap_length: Accessor<'a, CapabilityRegistersLength>,
    pub hcs_params_1: Accessor<'a, StructuralParameters1>,
    pub hc_cp_params_1: Accessor<'a, HCCapabilityParameters1>,
    pub rts_off: Accessor<'a, RuntimeRegisterSpaceOffset>,
}

impl<'a> HCCapabilityRegisters<'a> {
    pub fn new(mmio_base: PhysAddr) -> Self {
        let cap_length = Accessor::new(mmio_base, 0);
        let hcs_params_1 = Accessor::new(mmio_base, 0x04);
        let hc_cp_params_1 = Accessor::new(mmio_base, 0x10);
        let rts_off = Accessor::new(mmio_base, 0x18);

        let hci_version = Accessor::<'a, HCInterfaceVersionNumber>::new(mmio_base, 0x2);
        assert!(
            hci_version.get() >= 0x0900,
            "Invalid version: {:X}",
            hci_version.get()
        );

        Self {
            cap_length,
            hcs_params_1,
            hc_cp_params_1,
            rts_off,
        }
    }
}

#[repr(transparent)]
pub struct CapabilityRegistersLength(u8);

impl CapabilityRegistersLength {
    pub fn get(&self) -> usize {
        self.0 as _
    }
}

#[repr(transparent)]
pub struct HCInterfaceVersionNumber(u16);
impl HCInterfaceVersionNumber {
    fn get(&self) -> u16 {
        self.0
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct StructuralParameters1(u32);
    pub number_of_device_slots, _: 7, 0;
}

bitfield! {
    #[repr(transparent)]
    pub struct HCCapabilityParameters1(u32);
    pub xhci_extended_capabilities_pointer,_: 31,16;
}

#[repr(transparent)]
pub struct RuntimeRegisterSpaceOffset(u32);
impl RuntimeRegisterSpaceOffset {
    pub fn get(&self) -> u32 {
        self.0 & !0xf
    }
}
