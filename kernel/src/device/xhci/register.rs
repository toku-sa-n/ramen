// SPDX-License-Identifier: GPL-3.0-or-later

use bit::BitIndex;
use proc_macros::add_register_type;

add_register_type! {
    pub struct UsbLegacySupportCapability: u32{
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
