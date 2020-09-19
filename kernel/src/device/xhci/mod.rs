// SPDX-License-Identifier: GPL-3.0-or-later

mod register;

use {
    super::pci::config,
    register::{
        HCCapabilityParameters1, HccapabilityParameters1Field, UsbLegacySupportCapability,
        UsbLegacySupportCapabilityField,
    },
};

pub struct Xhci {
    config_space: config::Space,
}

impl Xhci {
    pub fn get_ownership_from_bios(&self) {
        type Param1 = HCCapabilityParameters1;
        type Param1Field = HccapabilityParameters1Field;
        type LegacySupport = UsbLegacySupportCapability;
        type LegacySupportField = UsbLegacySupportCapabilityField;

        info!("Getting ownership from BIOS...");
        let mmio_base = self.config_space.bar().base_addr();

        Param1::edit(mmio_base, |param1| {
            let capability_ptr = param1.get(Param1Field::XhciExtendedCapabilitiesPointer);
            let capability_base = mmio_base + (capability_ptr << 2) as usize;

            let bios_owns_semaphore = LegacySupportField::HcBiosOwnedSemaphore;
            let os_owns_semaphore = LegacySupportField::HcOsOwnedSemaphore;

            LegacySupport::edit(capability_base, |legacy_support| {
                legacy_support.set(bios_owns_semaphore, 1);

                while {
                    let bios_owns = legacy_support.get(bios_owns_semaphore) == 0;
                    let os_owns = legacy_support.get(os_owns_semaphore) == 1;

                    os_owns && !bios_owns
                } {}
            })
        });
        info!("Done");
    }

    fn new(config_space: config::Space) -> Result<Self, Error> {
        if config_space.is_xhci() {
            info!("base: {:X}", &config_space.bar().base_addr().as_u64());
            Ok(Self { config_space })
        } else {
            Err(Error::NotXhciDevice)
        }
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
