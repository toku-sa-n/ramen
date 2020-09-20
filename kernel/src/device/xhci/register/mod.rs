// SPDX-License-Identifier: GPL-3.0-or-later

use {bit::BitIndex, proc_macros::add_register_type};

pub mod hc_capability_registers;
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
    pub struct ConfigureRegister:u32{
        max_device_slots_enabled:0..8,
    }
}
