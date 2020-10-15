// SPDX-License-Identifier: GPL-3.0-or-later

mod event_ring;
mod extended_capability;
mod register;
mod transfer_ring;

use {
    super::config::{self, bar},
    crate::{
        accessor::slice,
        mem::{allocator::phys::FRAME_MANAGER, paging::pml4::PML4},
    },
    core::convert::TryFrom,
    os_units::Size,
    register::{
        hc_capability_registers::HCCapabilityRegisters,
        hc_operational_registers::HCOperationalRegisters,
        runtime_base_registers::RuntimeBaseRegisters,
        usb_legacy_support_capability::UsbLegacySupportCapability,
    },
    transfer_ring::{Command, Event, RingQueue},
    x86_64::{
        structures::paging::{FrameAllocator, MapperAllSizes},
        PhysAddr,
    },
};

pub struct Xhci<'a> {
    usb_legacy_support_capability: Option<UsbLegacySupportCapability<'a>>,
    hc_capability_registers: HCCapabilityRegisters<'a>,
    hc_operational_registers: HCOperationalRegisters<'a>,
    dcbaa: DeviceContextBaseAddressArray<'a>,
    command_ring: RingQueue<Command>,
    event_ring: RingQueue<Event>,
    runtime_base_registers: RuntimeBaseRegisters<'a>,
    event_ring_segment_table: event_ring::SegmentTable<'a>,
    config_space: config::Space,
}

impl<'a> Xhci<'a> {
    pub fn init(&mut self) {
        self.get_ownership_from_bios();
        self.reset_hc();
        self.wait_until_hc_is_ready();
        self.set_num_of_enabled_slots();
        self.set_dcbaap();
        self.set_command_ring_pointer();
        self.init_msi();
        self.set_event_ring_dequeue_pointer();
        self.enable_system_bus_interrupt_generation();
        self.init_event_ring_segment_table();
        self.enable_interrupt();
        self.run();
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
            .set_num_of_device_slots(num_of_slots);
    }

    fn set_dcbaap(&mut self) {
        info!("Set DCBAAP...");
        self.hc_operational_registers.set_dcbaa_ptr(self.dcbaa.phys);
    }

    fn set_command_ring_pointer(&mut self) {
        info!("Setting command ring pointer...");
        let virt_addr = self.command_ring.addr();
        let phys_addr = PML4.lock().translate_addr(virt_addr).unwrap();

        self.hc_operational_registers
            .set_command_ring_ptr(phys_addr);
    }

    fn init_msi(&mut self) {
        info!("Initializing MSI...");
        self.config_space.init_msi_for_xhci().unwrap();
    }

    fn init_event_ring_segment_table(&mut self) {
        info!("Initializing event ring segment table...");
        let ring_addr = self.event_ring.addr();
        let phys = PML4.lock().translate_addr(ring_addr).unwrap();
        self.event_ring_segment_table.edit(|table| {
            table[0].set_base_address(phys);
            table[0].set_segment_size(256);
        });

        self.set_event_ring_segment_table_size();
        self.set_event_ring_segment_table_address();
    }

    fn set_event_ring_segment_table_size(&mut self) {
        self.runtime_base_registers
            .set_event_ring_segment_table_size(
                u16::try_from(event_ring::NUM_ELEMENTS_SEGMENT_TABLE).unwrap(),
            )
    }

    fn set_event_ring_segment_table_address(&mut self) {
        self.runtime_base_registers
            .set_event_ring_segment_table_addr(self.event_ring_segment_table.addr())
    }

    fn set_event_ring_dequeue_pointer(&mut self) {
        let phys = PML4.lock().translate_addr(self.event_ring.addr()).unwrap();
        self.runtime_base_registers.set_event_ring_dequeue_ptr(phys)
    }

    fn enable_system_bus_interrupt_generation(&mut self) {
        self.hc_operational_registers.enable_interrupt()
    }

    fn enable_interrupt(&mut self) {
        self.runtime_base_registers.enable_interrupt()
    }

    fn run(&mut self) {
        self.hc_operational_registers.run();
    }

    fn new(config_space: config::Space) -> Result<Self, Error> {
        if config_space.is_xhci() {
            Ok(Self::generate(config_space))
        } else {
            Err(Error::NotXhciDevice)
        }
    }

    fn generate(config_space: config::Space) -> Self {
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

        Self {
            usb_legacy_support_capability,
            hc_capability_registers,
            hc_operational_registers,
            dcbaa,
            command_ring: RingQueue::<Command>::new(),
            config_space,
            event_ring: RingQueue::<Event>::new(),
            runtime_base_registers,
            event_ring_segment_table: event_ring::SegmentTable::new(),
        }
    }
}

const MAX_DEVICE_SLOT: usize = 255;

struct DeviceContextBaseAddressArray<'a> {
    arr: slice::Accessor<'a, usize>,
    phys: PhysAddr,
}

impl<'a> DeviceContextBaseAddressArray<'a> {
    fn new() -> Self {
        let phys_frame = FRAME_MANAGER.lock().allocate_frame().unwrap();
        let phys = phys_frame.start_address();
        let arr = slice::Accessor::new(phys, Size::new(0), MAX_DEVICE_SLOT + 1);
        Self { arr, phys }
    }
}

#[derive(Debug)]
enum Error {
    NotXhciDevice,
}

pub fn iter_devices<'a>() -> impl Iterator<Item = Xhci<'a>> {
    super::iter_devices().filter_map(|device| {
        if device.is_xhci() {
            Xhci::new(device).ok()
        } else {
            None
        }
    })
}
