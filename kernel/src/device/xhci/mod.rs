// SPDX-License-Identifier: GPL-3.0-or-later

mod register;

use {
    super::pci::config,
    register::{
        hc_capability_registers::{
            CapabilityRegistersLength, CapabilityRegistersLengthField, HCCapabilityRegisters,
            StructuralParameters1Field,
        },
        usb_legacy_support_capability::{
            UsbLegacySupportCapability, UsbLegacySupportCapabilityRegister,
            UsbLegacySupportCapabilityRegisterField,
        },
        ConfigureRegister, ConfigureRegisterField, HCCapabilityParameters1,
        HccapabilityParameters1Field, UsbStatusRegister, UsbStatusRegisterField,
    },
    x86_64::PhysAddr,
};

pub struct Xhci {
    usb_legacy_support_capability: UsbLegacySupportCapability,
    usb_status_register: UsbStatusRegister,
    configure_register: ConfigureRegister,
    hc_capability_registers: HCCapabilityRegisters,
}

impl Xhci {
    pub fn init(&self) {
        self.get_ownership_from_bios();
        self.wait_until_controller_is_ready();
        self.set_num_of_enabled_slots();
    }

    fn get_ownership_from_bios(&self) {
        type LegacySupport = UsbLegacySupportCapabilityRegister;
        type LegacySupportField = UsbLegacySupportCapabilityRegisterField;

        info!("Getting ownership from BIOS...");

        let usb_leg_sup = &self.usb_legacy_support_capability.usb_leg_sup;

        let bios_owns_semaphore = LegacySupportField::HcBiosOwnedSemaphore;
        let os_owns_semaphore = LegacySupportField::HcOsOwnedSemaphore;

        usb_leg_sup.set(os_owns_semaphore, 1);

        while {
            let bios_owns = usb_leg_sup.get(bios_owns_semaphore) == 0;
            let os_owns = usb_leg_sup.get(os_owns_semaphore) == 1;

            os_owns && !bios_owns
        } {}

        info!("Done");
    }

    fn set_num_of_enabled_slots(&self) {
        info!("Setting the number of slots...");
        let num_of_slots = self
            .hc_capability_registers
            .hcs_params_1
            .get(StructuralParameters1Field::NumberOfDeviceSlots);

        self.configure_register
            .set(ConfigureRegisterField::MaxDeviceSlotsEnabled, num_of_slots);
        info!("Done.");
    }

    fn new(config_space: config::Space) -> Result<Self, Error> {
        if config_space.is_xhci() {
            info!("xHC found.");

            let mmio_base = config_space.bar().base_addr();
            let hc_capability_parameters1 = Self::fetch::<HCCapabilityParameters1>(mmio_base, 0x10);

            let xecp = hc_capability_parameters1
                .get(HccapabilityParameters1Field::XhciExtendedCapabilitiesPointer);
            let usb_legacy_support_capability =
                UsbLegacySupportCapability::new(mmio_base, xecp as usize);

            let hc_capability_registers = HCCapabilityRegisters::new(mmio_base);
            let operational_base = mmio_base
                + hc_capability_registers
                    .cap_length
                    .get(CapabilityRegistersLengthField::Len) as usize;

            let usb_status_register = Self::fetch::<UsbStatusRegister>(operational_base, 0x04);

            let configure_register = Self::fetch::<ConfigureRegister>(operational_base, 0x38);

            Ok(Self {
                usb_legacy_support_capability,
                usb_status_register,
                configure_register,
                hc_capability_registers,
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

    fn fetch<T: register::Register>(base: PhysAddr, offset: usize) -> T {
        info!("Fetching {}...", T::name());
        let r = T::new(base, offset);
        info!("Done.");
        r
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
