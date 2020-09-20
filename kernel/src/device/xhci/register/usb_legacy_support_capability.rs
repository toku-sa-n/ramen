// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::device::xhci::register::Register, bit::BitIndex, proc_macros::add_register_type,
    x86_64::PhysAddr,
};

pub struct UsbLegacySupportCapability {
    pub usb_leg_sup: UsbLegacySupportCapabilityRegister,
}

impl UsbLegacySupportCapability {
    pub fn new(mmio_base: PhysAddr, xecp: usize) -> Self {
        let base = mmio_base + (xecp << 2);
        let usb_leg_sup = UsbLegacySupportCapabilityRegister::new(base, 0);

        Self { usb_leg_sup }
    }
}

add_register_type! {
    pub struct UsbLegacySupportCapabilityRegister: u32{
        capability_id: 0..8,
        hc_bios_owned_semaphore: 16..17,
        hc_os_owned_semaphore: 24..25,
    }
}
