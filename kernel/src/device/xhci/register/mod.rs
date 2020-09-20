// SPDX-License-Identifier: GPL-3.0-or-later

use {bit::BitIndex, proc_macros::add_register_type};

pub mod usb_legacy_support_capability;

pub trait Register {
    fn name() -> &'static str;
    fn new(base: x86_64::PhysAddr, offset: usize) -> Self;
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
