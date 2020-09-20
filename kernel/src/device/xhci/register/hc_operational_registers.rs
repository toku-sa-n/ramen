// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::device::xhci::register::{
        hc_capability_registers::{CapabilityRegistersLength, CapabilityRegistersLengthField},
        Register,
    },
    bit::BitIndex,
    proc_macros::add_register_type,
    x86_64::PhysAddr,
};

pub struct HCOperationalRegisters {
    pub usb_sts: UsbStatusRegister,
    pub dcbaap: DeviceContextBaseAddressArrayPointer,
    pub config: ConfigureRegister,
}

impl HCOperationalRegisters {
    pub fn new(mmio_base: PhysAddr, cap_length: &CapabilityRegistersLength) -> Self {
        let operational_base =
            mmio_base + cap_length.get(CapabilityRegistersLengthField::Len) as usize;

        let usb_sts = UsbStatusRegister::new(operational_base, 0x04);
        let dcbaap = DeviceContextBaseAddressArrayPointer::new(operational_base, 0x30);
        let config = ConfigureRegister::new(operational_base, 0x38);

        Self {
            usb_sts,
            dcbaap,
            config,
        }
    }
}

add_register_type! {
    pub struct UsbStatusRegister:u32{
        controller_not_ready:11..12,
    }
}

add_register_type! {
    pub struct ConfigureRegister:u32{
        max_device_slots_enabled:0..8,
    }
}

add_register_type! {
    pub struct DeviceContextBaseAddressArrayPointer:u64{
        pointer:6..64,
    }
}
