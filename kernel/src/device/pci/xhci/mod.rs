// SPDX-License-Identifier: GPL-3.0-or-later

mod register;
mod ring;

use {
    super::config::{self, bar},
    crate::mem::allocator::page_box::PageBox,
    futures_util::{task::AtomicWaker, StreamExt},
    register::{hc_capability_registers::NumberOfDeviceSlots, Registers},
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
    dcbaa: DeviceContextBaseAddressArray,
    command_ring: command::Ring,
    event_ring: event::Ring,
    registers: Registers,
}

impl<'a> Xhci {
    fn init(&mut self) {
        self.get_ownership_from_bios();
        self.reset();
        self.wait_until_ready();
        self.set_num_of_enabled_slots();
        self.set_dcbaap();
        self.set_command_ring_pointer();
        self.set_event_ring_dequeue_pointer();
        self.init_event_ring_segment_table();
        self.run();

        self.issue_noop();
    }

    fn get_ownership_from_bios(&mut self) {
        self.registers.transfer_hc_ownership_to_os();
    }

    fn reset(&mut self) {
        self.registers.reset_hc()
    }

    fn wait_until_ready(&self) {
        self.registers.wait_until_hc_is_ready();
    }

    fn set_num_of_enabled_slots(&mut self) {
        self.registers.init_num_of_slots()
    }

    fn set_dcbaap(&mut self) {
        self.registers.set_dcbaap(self.dcbaa.phys_addr())
    }

    fn set_command_ring_pointer(&mut self) {
        self.registers
            .set_command_ring_pointer(self.command_ring.phys_addr())
    }

    fn init_event_ring_segment_table(&mut self) {
        self.set_event_ring_segment_table_size();
        self.enable_event_ring()
    }

    fn enable_event_ring(&mut self) {
        self.set_event_ring_segment_table_address()
    }

    fn set_event_ring_segment_table_size(&mut self) {
        self.registers.set_event_ring_segment_table_size();
    }

    fn set_event_ring_segment_table_address(&mut self) {
        self.registers
            .set_event_ring_segment_table_addr(self.event_ring.phys_addr_to_segment_table())
    }

    fn set_event_ring_dequeue_pointer(&mut self) {
        self.registers
            .set_event_ring_dequeue_pointer(self.event_ring.phys_addr_to_array_beginning())
    }

    fn run(&mut self) {
        self.registers.run_hc()
    }

    fn issue_noop(&mut self) {
        self.command_ring.send_noop();
        self.registers.notify_to_hc();
    }

    fn new(config_space: &config::Space) -> Result<Self, Error> {
        if config_space.is_xhci() {
            Ok(Self::generate(&config_space))
        } else {
            Err(Error::NotXhciDevice)
        }
    }

    fn generate(config_space: &config::Space) -> Self {
        let mmio_base = config_space.base_address(bar::Index::new(0));
        let registers = Registers::new(mmio_base);
        let dcbaa = DeviceContextBaseAddressArray::new(registers.num_of_device_slots());
        let max_num_of_erst = registers.max_num_of_erst();
        Self {
            registers,
            dcbaa,
            command_ring: command::Ring::new(),
            event_ring: event::Ring::new(max_num_of_erst),
        }
    }
}

struct DeviceContextBaseAddressArray {
    arr: PageBox<[usize]>,
}
impl DeviceContextBaseAddressArray {
    fn new(number_of_slots: NumberOfDeviceSlots) -> Self {
        let number_of_slots: u8 = number_of_slots.into();
        let number_of_slots: usize = number_of_slots.into();
        let arr = PageBox::new_slice(number_of_slots + 1);
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
