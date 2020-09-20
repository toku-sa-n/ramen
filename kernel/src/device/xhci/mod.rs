// SPDX-License-Identifier: GPL-3.0-or-later

mod register;

use {
    super::pci::config,
    crate::mem::{
        allocator::{phys::FRAME_MANAGER, virt},
        paging::pml4::PML4,
    },
    core::slice,
    register::{
        hc_capability_registers::{HCCapabilityRegisters, StructuralParameters1Field},
        hc_operational_registers::{
            ConfigureRegisterField, HCOperationalRegisters, UsbStatusRegisterField,
        },
        usb_legacy_support_capability::{
            UsbLegacySupportCapability, UsbLegacySupportCapabilityRegisterField,
        },
    },
    x86_64::{
        structures::paging::{FrameAllocator, Mapper, PageTableFlags},
        VirtAddr,
    },
};

pub struct Xhci {
    usb_legacy_support_capability: UsbLegacySupportCapability,
    hc_capability_registers: HCCapabilityRegisters,
    hc_operational_registers: HCOperationalRegisters,
}

impl Xhci {
    pub fn init(&self) {
        self.get_ownership_from_bios();
        self.wait_until_controller_is_ready();
        self.set_num_of_enabled_slots();
    }

    fn get_ownership_from_bios(&self) {
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

        self.hc_operational_registers
            .config
            .set(ConfigureRegisterField::MaxDeviceSlotsEnabled, num_of_slots);
        info!("Done.");
    }

    fn new(config_space: &config::Space) -> Result<Self, Error> {
        if config_space.is_xhci() {
            info!("xHC found.");

            let mmio_base = config_space.bar().base_addr();

            let hc_capability_registers = HCCapabilityRegisters::new(mmio_base);
            let usb_legacy_support_capability =
                UsbLegacySupportCapability::new(mmio_base, &hc_capability_registers);

            let hc_operational_registers =
                HCOperationalRegisters::new(mmio_base, &hc_capability_registers.cap_length);

            Ok(Self {
                usb_legacy_support_capability,
                hc_capability_registers,
                hc_operational_registers,
            })
        } else {
            Err(Error::NotXhciDevice)
        }
    }

    fn wait_until_controller_is_ready(&self) {
        info!("Waiting until controller is ready...");
        while self
            .hc_operational_registers
            .usb_sts
            .get(UsbStatusRegisterField::ControllerNotReady)
            == 1
        {}
        info!("Controller is ready");
    }
}

const MAX_DEVICE_SLOT: usize = 255;

struct DeviceContextBaseAddressArray([usize; MAX_DEVICE_SLOT]);

impl DeviceContextBaseAddressArray {
    fn new(num_of_enabled_slots: usize) -> Self {
        Self([0; MAX_DEVICE_SLOT])
    }
}

#[derive(Debug)]
enum Error {
    NotXhciDevice,
}

pub fn iter_devices() -> impl Iterator<Item = Xhci> {
    super::pci::iter_devices().filter_map(|device| {
        if device.is_xhci() {
            Xhci::new(&device).ok()
        } else {
            None
        }
    })
}
