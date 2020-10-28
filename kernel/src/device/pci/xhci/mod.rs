// SPDX-License-Identifier: GPL-3.0-or-later

mod register;
mod ring;

use {
    super::config::{self, bar},
    crate::mem::allocator::page_box::PageBox,
    futures_util::{task::AtomicWaker, StreamExt},
    register::{
        doorbell, hc_capability_registers::HCCapabilityRegisters,
        hc_operational_registers::HCOperationalRegisters,
        runtime_base_registers::RuntimeBaseRegisters,
        usb_legacy_support_capability::UsbLegacySupportCapability,
    },
    ring::{command, event},
    x86_64::PhysAddr,
};

static WAKER: AtomicWaker = AtomicWaker::new();

pub async fn task() {
    let mut xhci = iter_devices().next().unwrap();
    xhci.init();

    while let Some(trb) = xhci.event_ring.next().await {
        info!("TRB: {:?}", trb);
    }
}

pub struct Xhci {
    usb_legacy_support_capability: Option<UsbLegacySupportCapability>,
    hc_capability_registers: HCCapabilityRegisters,
    hc_operational_registers: HCOperationalRegisters,
    dcbaa: DeviceContextBaseAddressArray,
    command_ring: command::Ring,
    event_ring: event::Ring,
    runtime_base_registers: RuntimeBaseRegisters,
    doorbell_array: doorbell::Array,
}

impl<'a> Xhci {
    fn init(&mut self) {
        self.get_ownership_from_bios();
        self.reset_hc();
        self.wait_until_hc_is_ready();
        self.set_num_of_enabled_slots();
        self.set_dcbaap();
        self.set_command_ring_pointer();
        self.set_event_ring_dequeue_pointer();
        self.init_event_ring_segment_table();
        self.run();

        self.issue_noop();
    }

    fn get_ownership_from_bios(&mut self) {
        if let Some(ref mut usb_leg_sup_cap) = self.usb_legacy_support_capability {
            info!("Getting ownership from BIOS...");
            usb_leg_sup_cap.give_hc_ownership_to_os();
        }
    }

    fn reset_hc(&mut self) {
        self.hc_operational_registers.reset_hc();
        info!("Reset completed.");
    }

    fn wait_until_hc_is_ready(&self) {
        self.hc_operational_registers.wait_until_hc_is_ready();
    }

    fn set_num_of_enabled_slots(&mut self) {
        let num_of_slots = self.hc_capability_registers.number_of_device_slots();

        self.hc_operational_registers
            .set_num_of_device_slots(num_of_slots.into());
    }

    fn set_dcbaap(&mut self) {
        info!("Set DCBAAP...");
        self.hc_operational_registers
            .set_dcbaa_ptr(self.dcbaa.phys_addr());
    }

    fn set_command_ring_pointer(&mut self) {
        info!("Setting command ring pointer...");
        self.hc_operational_registers
            .set_command_ring_ptr(self.command_ring.phys_addr());
    }

    fn init_event_ring_segment_table(&mut self) {
        self.set_event_ring_segment_table_size();
        self.enable_event_ring()
    }

    fn enable_event_ring(&mut self) {
        self.set_event_ring_segment_table_address()
    }

    fn set_event_ring_segment_table_size(&mut self) {
        let max_num_of_erst = self.hc_capability_registers.max_num_of_erst();
        self.runtime_base_registers
            .set_event_ring_segment_table_size(max_num_of_erst.into());
    }

    fn set_event_ring_segment_table_address(&mut self) {
        self.runtime_base_registers
            .set_event_ring_segment_table_addr(self.event_ring.phys_addr_to_segment_table())
    }

    fn set_event_ring_dequeue_pointer(&mut self) {
        self.runtime_base_registers
            .set_event_ring_dequeue_ptr(self.event_ring.phys_addr_to_array_beginning())
    }

    fn run(&mut self) {
        self.hc_operational_registers.run();
    }

    fn issue_noop(&mut self) {
        self.command_ring.send_noop();
        self.doorbell_array.notify_to_hc();
    }

    fn new(config_space: &config::Space) -> Result<Self, Error> {
        if config_space.is_xhci() {
            Ok(Self::generate(&config_space))
        } else {
            Err(Error::NotXhciDevice)
        }
    }

    fn generate(config_space: &config::Space) -> Self {
        info!("xHC found.");

        let mmio_base = config_space.base_address(bar::Index::new(0));

        info!("Getting HCCapabilityRegisters...");
        let hc_capability_registers = HCCapabilityRegisters::new(mmio_base);

        info!("Getting UsbLegacySupportCapability...");
        let usb_legacy_support_capability =
            UsbLegacySupportCapability::new(mmio_base, &hc_capability_registers);

        info!("Getting HCOperationalRegisters...");
        let hc_operational_registers =
            HCOperationalRegisters::new(mmio_base, &hc_capability_registers);

        info!("Getting DCBAA...");
        let dcbaa = DeviceContextBaseAddressArray::new();

        let runtime_base_registers = RuntimeBaseRegisters::new(
            mmio_base,
            hc_capability_registers.offset_to_runtime_registers() as usize,
        );

        let doorbell_array = doorbell::Array::new(mmio_base, hc_capability_registers.db_off());
        let max_num_of_erst = hc_capability_registers.max_num_of_erst();

        Self {
            usb_legacy_support_capability,
            hc_capability_registers,
            hc_operational_registers,
            dcbaa,
            command_ring: command::Ring::new(256),
            event_ring: event::Ring::new(max_num_of_erst),
            runtime_base_registers,
            doorbell_array,
        }
    }
}

const MAX_DEVICE_SLOT: usize = 255;

struct DeviceContextBaseAddressArray {
    arr: PageBox<[usize]>,
}
impl DeviceContextBaseAddressArray {
    fn new() -> Self {
        let arr = PageBox::new_slice(MAX_DEVICE_SLOT + 1);
        Self { arr }
    }

    fn phys_addr(&self) -> PhysAddr {
        self.arr.phys_addr()
    }
}

#[derive(Debug)]
enum Error {
    NotXhciDevice,
}

pub fn iter_devices() -> impl Iterator<Item = Xhci> {
    super::iter_devices().filter_map(|device| {
        if device.is_xhci() {
            Xhci::new(&device).ok()
        } else {
            None
        }
    })
}
