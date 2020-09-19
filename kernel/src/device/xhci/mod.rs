// SPDX-License-Identifier: GPL-3.0-or-later

mod register;

use {
    super::pci::config,
    register::{
        CapabilityRegistersLength, HCCapabilityParameters1, HccapabilityParameters1Field,
        UsbLegacySupportCapability, UsbLegacySupportCapabilityField,
    },
    x86_64::PhysAddr,
};

pub struct Xhci {
    config_space: config::Space,
    hc_capability_parameters1: HCCapabilityParameters1,
    usb_legacy_support_capability: UsbLegacySupportCapability,
    capability_registers_length: CapabilityRegistersLength,
}

impl Xhci {
    pub fn get_ownership_from_bios(&self) {
        type LegacySupport = UsbLegacySupportCapability;
        type LegacySupportField = UsbLegacySupportCapabilityField;

        info!("Getting ownership from BIOS...");

        let bios_owns_semaphore = LegacySupportField::HcBiosOwnedSemaphore;
        let os_owns_semaphore = LegacySupportField::HcOsOwnedSemaphore;

        self.usb_legacy_support_capability.set(os_owns_semaphore, 1);

        while {
            let bios_owns = self.usb_legacy_support_capability.get(bios_owns_semaphore) == 0;
            let os_owns = self.usb_legacy_support_capability.get(os_owns_semaphore) == 1;

            os_owns && !bios_owns
        } {}

        info!("Done");
    }

    fn new(config_space: config::Space) -> Result<Self, Error> {
        if config_space.is_xhci() {
            info!("xHC found.");

            let mmio_base = config_space.bar().base_addr();
            let hc_capability_parameters1 = Self::fetch_hc_capability_parameters1(mmio_base);

            let capability_ptr = hc_capability_parameters1
                .get(HccapabilityParameters1Field::XhciExtendedCapabilitiesPointer);
            let capability_base = mmio_base + (capability_ptr << 2) as usize;
            let usb_legacy_support_capability =
                Self::fetch_usb_legacy_support_capability(capability_base);

            let capability_registers_length = Self::fetch_capability_registers_length(mmio_base);

            Ok(Self {
                config_space,
                hc_capability_parameters1,
                usb_legacy_support_capability,
                capability_registers_length,
            })
        } else {
            Err(Error::NotXhciDevice)
        }
    }

    fn fetch_hc_capability_parameters1(mmio_base: PhysAddr) -> HCCapabilityParameters1 {
        info!("Fetching HCCapabilityParameters1...");
        let hc_capability_parameters1 = HCCapabilityParameters1::new(mmio_base + 0x10usize);
        info!("Done.");
        hc_capability_parameters1
    }

    fn fetch_capability_registers_length(mmio_base: PhysAddr) -> CapabilityRegistersLength {
        info!("Fetching CapabilityRegistersLength...");
        let len = CapabilityRegistersLength::new(mmio_base);
        info!("Done.");
        len
    }

    fn fetch_usb_legacy_support_capability(
        capability_base: PhysAddr,
    ) -> UsbLegacySupportCapability {
        info!("Fetching UsbLegacySupportCapability...");
        let usb_legacy_support_capability = UsbLegacySupportCapability::new(capability_base);
        info!("Done.");
        usb_legacy_support_capability
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
