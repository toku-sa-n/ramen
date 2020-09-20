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
        hc_capability_registers::HCCapabilityRegisters,
        hc_operational_registers::HCOperationalRegisters,
        usb_legacy_support_capability::UsbLegacySupportCapability,
    },
    x86_64::{
        structures::paging::{FrameAllocator, Mapper, MapperAllSizes, PageTableFlags},
        VirtAddr,
    },
};

pub struct Xhci {
    usb_legacy_support_capability: UsbLegacySupportCapability,
    hc_capability_registers: HCCapabilityRegisters,
    hc_operational_registers: HCOperationalRegisters,
    dcbaa: DeviceContextBaseAddressArray,
}

impl Xhci {
    pub fn init(&self) {
        self.get_ownership_from_bios();
        self.wait_until_controller_is_ready();
        self.set_num_of_enabled_slots();
        self.set_dcbaap();
    }

    fn get_ownership_from_bios(&self) {
        info!("Getting ownership from BIOS...");

        let usb_leg_sup = &self.usb_legacy_support_capability.usb_leg_sup;

        usb_leg_sup.set_hc_os_owned_semaphore(1);

        while {
            let bios_owns = usb_leg_sup.get_hc_bios_owned_semaphore() == 0;
            let os_owns = usb_leg_sup.get_hc_os_owned_semaphore() == 1;

            os_owns && !bios_owns
        } {}

        info!("Done");
    }

    fn wait_until_controller_is_ready(&self) {
        info!("Waiting until controller is ready...");
        while self
            .hc_operational_registers
            .usb_sts
            .get_controller_not_ready()
            == 1
        {}
        info!("Controller is ready");
    }

    fn set_num_of_enabled_slots(&self) {
        info!("Setting the number of slots...");
        let num_of_slots = self
            .hc_capability_registers
            .hcs_params_1
            .get_number_of_device_slots();

        self.hc_operational_registers
            .config
            .set_max_device_slots_enabled(num_of_slots);
        info!("Done.");
    }

    fn set_dcbaap(&self) {
        info!("Set DCBAAP...");
        let phys_addr_of_dcbaa = PML4
            .lock()
            .translate_addr(VirtAddr::new(&self.dcbaa as *const _ as u64))
            .expect("Failed to fetch the physical address of DCBAA");

        self.hc_operational_registers
            .dcbaap
            .set_pointer(phys_addr_of_dcbaa.as_u64() >> 6);
        info!("Done.");
    }

    fn new(config_space: &config::Space) -> Result<Self, Error> {
        if config_space.is_xhci() {
            info!("xHC found.");

            let mmio_base = config_space.bar().base_addr();

            info!("Getting HCCapabilityRegisters...");
            let hc_capability_registers = HCCapabilityRegisters::new(mmio_base);
            info!("Done.");

            info!("Getting UsbLegacySupportCapability...");
            let usb_legacy_support_capability =
                UsbLegacySupportCapability::new(mmio_base, &hc_capability_registers);
            info!("Done.");

            info!("Getting HCOperationalRegisters...");
            let hc_operational_registers =
                HCOperationalRegisters::new(mmio_base, &hc_capability_registers.cap_length);
            info!("Done.");

            info!("Getting DCBAA...");
            let dcbaa = DeviceContextBaseAddressArray::new();
            info!("Done.");

            Ok(Self {
                usb_legacy_support_capability,
                hc_capability_registers,
                hc_operational_registers,
                dcbaa,
            })
        } else {
            Err(Error::NotXhciDevice)
        }
    }
}

const MAX_DEVICE_SLOT: usize = 255;

struct DeviceContextBaseAddressArray([usize; MAX_DEVICE_SLOT]);

impl DeviceContextBaseAddressArray {
    fn new() -> Self {
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
