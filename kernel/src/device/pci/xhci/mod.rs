// SPDX-License-Identifier: GPL-3.0-or-later

mod register;
mod ring;

use {
    super::config::bar,
    crate::mem::allocator::page_box::PageBox,
    futures_util::{task::AtomicWaker, StreamExt},
    register::{hc_capability_registers::NumberOfDeviceSlots, Registers},
    ring::{command, event},
    spinning_top::Spinlock,
    x86_64::PhysAddr,
};

static WAKER: AtomicWaker = AtomicWaker::new();

pub async fn task() {
    let registers = Spinlock::new(iter_devices().next().unwrap());
    let mut xhci = Xhci::new(&registers);
    let mut event_ring = event::Ring::new(&registers);
    xhci.init();

    while let Some(trb) = event_ring.next().await {
        info!("TRB: {:?}", trb);
    }
}

pub struct Xhci<'a> {
    dcbaa: DeviceContextBaseAddressArray,
    command_ring: command::Ring,
    registers: &'a Spinlock<Registers>,
}

impl<'a> Xhci<'a> {
    fn init(&mut self) {
        self.get_ownership_from_bios();
        self.reset();
        self.wait_until_ready();
        self.set_num_of_enabled_slots();
        self.set_dcbaap();
        self.set_command_ring_pointer();
        self.run();

        self.issue_noop();
    }

    fn get_ownership_from_bios(&mut self) {
        self.registers.lock().transfer_hc_ownership_to_os();
    }

    fn reset(&mut self) {
        self.registers.lock().reset_hc()
    }

    fn wait_until_ready(&self) {
        self.registers.lock().wait_until_hc_is_ready();
    }

    fn set_num_of_enabled_slots(&mut self) {
        self.registers.lock().init_num_of_slots()
    }

    fn set_dcbaap(&mut self) {
        self.registers.lock().set_dcbaap(self.dcbaa.phys_addr())
    }

    fn set_command_ring_pointer(&mut self) {
        self.registers
            .lock()
            .set_command_ring_pointer(self.command_ring.phys_addr())
    }

    fn run(&mut self) {
        self.registers.lock().run_hc()
    }

    fn issue_noop(&mut self) {
        self.command_ring.send_noop();
        self.registers.lock().notify_to_hc();
    }

    fn new(registers: &'a Spinlock<Registers>) -> Self {
        let dcbaa = DeviceContextBaseAddressArray::new(registers.lock().num_of_device_slots());
        Self {
            registers,
            dcbaa,
            command_ring: command::Ring::new(),
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

pub fn iter_devices() -> impl Iterator<Item = Registers> {
    super::iter_devices().filter_map(|device| {
        if device.is_xhci() {
            Some(Registers::new(device.base_address(bar::Index::new(0))))
        } else {
            None
        }
    })
}
