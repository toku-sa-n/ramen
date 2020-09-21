// SPDX-License-Identifier: GPL-3.0-or-later

mod register;
mod transfer_ring;

use {
    super::pci::config,
    crate::mem::paging::pml4::PML4,
    register::{
        hc_capability_registers::HCCapabilityRegisters,
        hc_operational_registers::HCOperationalRegisters,
        usb_legacy_support_capability::UsbLegacySupportCapability,
    },
    transfer_ring::{transfer_request_block::Command, RingQueue},
    x86_64::{structures::paging::MapperAllSizes, VirtAddr},
};

pub struct Xhci<'a> {
    usb_legacy_support_capability: UsbLegacySupportCapability<'a>,
    hc_capability_registers: HCCapabilityRegisters<'a>,
    hc_operational_registers: HCOperationalRegisters<'a>,
    dcbaa: DeviceContextBaseAddressArray,
    command_ring: RingQueue<'a, Command>,
}

impl<'a> Xhci<'a> {
    pub fn init(&mut self) {
        self.get_ownership_from_bios();
        self.wait_until_controller_is_ready();
        self.set_num_of_enabled_slots();
        self.set_dcbaap();
    }

    fn get_ownership_from_bios(&mut self) {
        info!("Getting ownership from BIOS...");

        let usb_leg_sup = &mut self.usb_legacy_support_capability.usb_leg_sup;

        usb_leg_sup.request_hc_ownership(true);

        while {
            let bios_owns = usb_leg_sup.bios_owns_hc();
            let os_owns = usb_leg_sup.os_owns_hc();

            os_owns && !bios_owns
        } {}

        info!("Done");
    }

    fn wait_until_controller_is_ready(&self) {
        info!("Waiting until controller is ready...");
        while self.hc_operational_registers.usb_sts.controller_not_ready() {}
        info!("Controller is ready");
    }

    fn set_num_of_enabled_slots(&mut self) {
        info!("Setting the number of slots...");
        let num_of_slots = self
            .hc_capability_registers
            .hcs_params_1
            .number_of_device_slots();

        self.hc_operational_registers
            .config
            .set_max_device_slots_enabled(num_of_slots);
        info!("Done.");
    }

    fn set_dcbaap(&mut self) {
        info!("Set DCBAAP...");
        let phys_addr_of_dcbaa = PML4
            .lock()
            .translate_addr(VirtAddr::new(&self.dcbaa as *const _ as u64))
            .expect("Failed to fetch the physical address of DCBAA");

        self.hc_operational_registers
            .dcbaap
            .set_ptr(phys_addr_of_dcbaa);
        info!("Done.");
    }

    fn set_command_ring_pointer(&mut self) {
        let virt_addr = self.command_ring.addr();
        let phys_addr = PML4.lock().translate_addr(virt_addr).unwrap();

        self.hc_operational_registers.crcr.set_ptr(phys_addr);
    }

    fn new(config_space: &config::Space) -> Result<Self, Error> {
        if config_space.is_xhci() {
            info!("xHC found.");

            let mmio_base = config_space.bar().base_addr();

            info!("Getting HCCapabilityRegisters...");
            let mut hc_capability_registers = HCCapabilityRegisters::new(mmio_base);
            info!("Done.");

            info!("Getting UsbLegacySupportCapability...");
            let usb_legacy_support_capability =
                UsbLegacySupportCapability::new(mmio_base, &hc_capability_registers);
            info!("Done.");

            info!("Getting HCOperationalRegisters...");
            let hc_operational_registers =
                HCOperationalRegisters::new(mmio_base, &mut hc_capability_registers.cap_length);
            info!("Done.");

            info!("Getting DCBAA...");
            let dcbaa = DeviceContextBaseAddressArray::new();
            info!("Done.");

            Ok(Self {
                usb_legacy_support_capability,
                hc_capability_registers,
                hc_operational_registers,
                dcbaa,
                command_ring: RingQueue::new(),
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

pub fn iter_devices<'a>() -> impl Iterator<Item = Xhci<'a>> {
    super::pci::iter_devices().filter_map(|device| {
        if device.is_xhci() {
            Xhci::new(&device).ok()
        } else {
            None
        }
    })
}
