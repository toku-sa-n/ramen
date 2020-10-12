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
    usb_leg_sup: Accessor<'a, UsbLegacySupportCapabilityRegister>,
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

    pub fn give_hc_ownership_to_os(&mut self) {
        let usb_leg_sup = &mut self.usb_leg_sup;
        usb_leg_sup.request_hc_ownership();

        while !usb_leg_sup.ownership_passed() {}
    }
}

bitfield! {
    #[repr(transparent)]
    struct UsbLegacySupportCapabilityRegister(u32);

    id, _: 7, 0;
    bios_owns_hc, _: 16;
    os_owns_hc, os_request_ownership: 24;
}
impl UsbLegacySupportCapabilityRegister {
    fn request_hc_ownership(&mut self) {
        self.os_request_ownership(true);
    }

    fn ownership_passed(&self) -> bool {
        !self.bios_owns_hc() && self.os_owns_hc()
    }
}
