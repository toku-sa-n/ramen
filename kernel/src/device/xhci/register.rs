// SPDX-License-Identifier: GPL-3.0-or-later

use {bit::BitIndex, proc_macros::add_register_type, x86_64::PhysAddr};

pub trait Register {
    fn name() -> &'static str;
    fn new(base: x86_64::PhysAddr, offset: usize) -> Self;
}

add_register_type! {
    pub struct UsbLegacySupportCapabilityRegister: u32{
        capability_id: 0..8,
        hc_bios_owned_semaphore: 16..17,
        hc_os_owned_semaphore: 24..25,
    }
}

add_register_type! {
    pub struct HCCapabilityParameters1:u32{
        xhci_extended_capabilities_pointer:16..32,
    }
}

add_register_type! {
    pub struct UsbStatusRegister:u32{
        controller_not_ready:11..12,
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

add_register_type! {
    pub struct ConfigureRegister:u32{
        max_device_slots_enabled:0..8,
    }
}

pub(super) struct UsbLegacySupportCapability {
    pub usb_leg_sup: UsbLegacySupportCapabilityRegister,
}

impl UsbLegacySupportCapability {
    pub fn new(mmio_base: PhysAddr, xecp: usize) -> Self {
        let base = mmio_base + (xecp << 2);
        let usb_leg_sup = UsbLegacySupportCapabilityRegister::new(base, 0);

        Self { usb_leg_sup }
    }
}
