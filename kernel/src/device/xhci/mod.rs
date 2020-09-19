// SPDX-License-Identifier: GPL-3.0-or-later

mod register;

use {
    super::pci::config,
    register::{
        CapabilityRegistersLength, CapabilityRegistersLengthField, HCCapabilityParameters1,
        HccapabilityParameters1Field, UsbLegacySupportCapability, UsbLegacySupportCapabilityField,
        UsbStatusRegister, UsbStatusRegisterField,
    },
    x86_64::PhysAddr,
};

pub struct Xhci {
    usb_legacy_support_capability: UsbLegacySupportCapability,
    usb_status_register: UsbStatusRegister,
}

impl Xhci {
    pub fn init(&self) {
        self.get_ownership_from_bios();
        self.wait_until_controller_is_ready();
    }

    fn get_ownership_from_bios(&self) {
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
            let operational_base = mmio_base
                + capability_registers_length.get(CapabilityRegistersLengthField::Len) as usize;

            let usb_status_register = Self::fetch_usb_status_register(operational_base);

            Ok(Self {
                usb_legacy_support_capability,
                usb_status_register,
            })
        } else {
            Err(Error::NotXhciDevice)
        }
    }

    fn wait_until_controller_is_ready(&self) {
        info!("Waiting until controller is ready...");
        while self
            .usb_status_register
            .get(UsbStatusRegisterField::ControllerNotReady)
            == 1
        {}
        info!("Controller is ready");
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

    fn fetch_usb_status_register(operational_base: PhysAddr) -> UsbStatusRegister {
        info!("Fetch UsbStatusRegister...");
        let status = UsbStatusRegister::new(operational_base + 0x04usize);
        info!("Done.");
        status
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
