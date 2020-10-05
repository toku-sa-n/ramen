// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{
        accessor::single_object::Accessor,
        device::pci::xhci::register::hc_capability_registers::HCCapabilityRegisters,
    },
    bitfield::bitfield,
    x86_64::PhysAddr,
};

pub struct UsbLegacySupportCapability<'a> {
    pub usb_leg_sup: Accessor<'a, UsbLegacySupportCapabilityRegister>,
}

impl<'a> UsbLegacySupportCapability<'a> {
    pub fn new(
        mmio_base: PhysAddr,
        hc_capability_registers: &HCCapabilityRegisters,
    ) -> Option<Self> {
        let xecp = hc_capability_registers
            .hc_cp_params_1
            .xhci_extended_capabilities_pointer();
        info!("xECP: {}", xecp);
        let base = mmio_base + ((xecp as usize) << 2);
        let usb_leg_sup = Accessor::<'a, UsbLegacySupportCapabilityRegister>::new(base, 0);

        if usb_leg_sup.id() == 1 {
            Some(Self { usb_leg_sup })
        } else {
            None
        }
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct UsbLegacySupportCapabilityRegister(u32);

    id, _: 7, 0;
    pub bios_owns_hc, _: 16;
    pub os_owns_hc, request_hc_ownership: 24;
}
