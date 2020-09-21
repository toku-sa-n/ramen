// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::device::xhci::register::{Accessor, Register},
    bitfield::bitfield,
    x86_64::PhysAddr,
};

pub struct HCCapabilityRegisters<'a> {
    pub cap_length: Accessor<'a, CapabilityRegistersLength>,
    pub hcs_params_1: Accessor<'a, StructuralParameters1>,
    pub hc_cp_params_1: Accessor<'a, HCCapabilityParameters1>,
}

impl<'a> HCCapabilityRegisters<'a> {
    pub fn new(mmio_base: PhysAddr) -> Self {
        let cap_length = Accessor::new(mmio_base, 0);
        let hcs_params_1 = Accessor::new(mmio_base, 0x04);
        let hc_cp_params_1 = Accessor::new(mmio_base, 0x10);

        Self {
            cap_length,
            hcs_params_1,
            hc_cp_params_1,
        }
    }
}

#[repr(transparent)]
pub struct CapabilityRegistersLength(u8);
impl Register for CapabilityRegistersLength {}

impl CapabilityRegistersLength {
    pub fn len(&mut self) -> usize {
        self.0 as _
    }
}

bitfield! {
    pub struct StructuralParameters1(u32);
    pub number_of_device_slots, _: 7, 0;
}
impl Register for StructuralParameters1 {}

bitfield! {
    pub struct HCCapabilityParameters1(u32);
    pub xhci_extended_capabilities_pointer,_: 31,16;
}
impl Register for HCCapabilityParameters1 {}
