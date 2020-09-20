// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::device::xhci::register::Register, bit::BitIndex, proc_macros::add_register_type,
    x86_64::PhysAddr,
};

pub struct HCCapabilityRegisters {
    pub cap_length: CapabilityRegistersLength,
    pub hcs_params_1: StructuralParameters1,
}

impl HCCapabilityRegisters {
    pub fn new(mmio_base: PhysAddr) -> Self {
        let cap_length = CapabilityRegistersLength::new(mmio_base, 0);
        let hcs_params_1 = StructuralParameters1::new(mmio_base, 0x04);

        Self {
            cap_length,
            hcs_params_1,
        }
    }
}

add_register_type! {
    pub struct CapabilityRegistersLength:u8{
        len:0..8,
    }
}

add_register_type! {
    pub struct StructuralParameters1:u32{
        number_of_device_slots:0..8,
    }
}
