// SPDX-License-Identifier: GPL-3.0-or-later

mod capability_register;
mod register;

use {
    super::pci::config,
    register::{
        HCCapabilityParameters1, HccapabilityParameters1Field, UsbLegacySupportCapability,
        UsbLegacySupportCapabilityField,
    },
    x86_64::PhysAddr,
};

pub struct Xhci {
    config_space: config::Space,
}

impl Xhci {
    fn new(config_space: config::Space) -> Result<Self, Error> {
        if config_space.is_xhci() {
            info!("base: {:X}", &config_space.bar().base_addr().as_u64());
            Ok(Self { config_space })
        } else {
            Err(Error::NotXhciDevice)
        }
    }

    pub fn get_ownership_from_bios(&self) {
        info!("Getting ownership from BIOS...");
        let mmio_base = self.config_space.bar().base_addr();

        let xhci_extended_capabilities_pointer = HCCapabilityParameters1::get(
            mmio_base,
            HccapabilityParameters1Field::XhciExtendedCapabilitiesPointer,
        );

        let usb_legacy_support_capability_base =
            mmio_base + (xhci_extended_capabilities_pointer << 2) as usize;
        UsbLegacySupportCapability::set(
            usb_legacy_support_capability_base,
            UsbLegacySupportCapabilityField::HcOsOwnedSemaphore,
            1,
        );

        while {
            let bios_owns = UsbLegacySupportCapability::get(
                usb_legacy_support_capability_base,
                UsbLegacySupportCapabilityField::HcBiosOwnedSemaphore,
            ) == 0;
            let os_owns = UsbLegacySupportCapability::get(
                usb_legacy_support_capability_base,
                UsbLegacySupportCapabilityField::HcOsOwnedSemaphore,
            ) == 1;

            os_owns && !bios_owns
        } {}
        info!("Done");
    }
}
#[derive(Debug)]
enum Error {
    NotXhciDevice,
}

pub fn iter_devices() -> impl Iterator<Item = Xhci> {
    super::pci::iter_devices().filter_map(|device| {
        if device.is_xhci() {
            Xhci::new(device).ok()
        } else {
            None
        }
    })
}
