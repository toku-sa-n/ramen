// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{
        accessor::single_object::Accessor,
        device::xhci::register::hc_capability_registers::HCCapabilityRegisters,
    },
    bitfield::bitfield,
    x86_64::PhysAddr,
};

pub struct UsbLegacySupportCapability<'a> {
    pub usb_leg_sup: Accessor<'a, UsbLegacySupportCapabilityRegister>,
}

impl<'a> UsbLegacySupportCapability<'a> {
    pub fn new(mmio_base: PhysAddr, hc_capability_registers: &HCCapabilityRegisters) -> Self {
        let xecp = hc_capability_registers
            .hc_cp_params_1
            .xhci_extended_capabilities_pointer();
        let base = mmio_base + (xecp << 2) as usize;
        let usb_leg_sup = Accessor::new(base, 0);

        Self { usb_leg_sup }
    }
}

bitfield! {
    pub struct UsbLegacySupportCapabilityRegister(u32);

    pub bios_owns_hc, _: 16;
    pub os_owns_hc, request_hc_ownership: 24;
}
