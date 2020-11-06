// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{
        device::pci::xhci::register::hc_capability::HCCapabilityRegisters, mem::accessor::Accessor,
    },
    bitfield::bitfield,
    os_units::Bytes,
    x86_64::PhysAddr,
};

pub struct UsbLegacySupportCapability {
    pub usb_leg_sup: Accessor<UsbLegacySupportCapabilityRegister>,
}

impl UsbLegacySupportCapability {
    pub fn new(
        mmio_base: PhysAddr,
        hc_capability_registers: &HCCapabilityRegisters,
    ) -> Option<Self> {
        let xecp = hc_capability_registers
            .hc_cp_params_1
            .read()
            .xhci_extended_capabilities_pointer();
        info!("xECP: {}", xecp);
        let base = mmio_base + ((xecp as usize) << 2);
        let usb_leg_sup = Accessor::<UsbLegacySupportCapabilityRegister>::new(base, Bytes::new(0));

        if usb_leg_sup.read().id() == 1 {
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
    pub os_owns_hc, os_request_ownership: 24;
}
