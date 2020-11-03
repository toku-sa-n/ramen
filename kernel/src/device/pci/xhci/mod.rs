// SPDX-License-Identifier: GPL-3.0-or-later

mod dcbaa;
mod register;
mod ring;

use {
    super::config::bar,
    dcbaa::DeviceContextBaseAddressArray,
    futures_util::{task::AtomicWaker, StreamExt},
    register::Registers,
    ring::{command, event},
    spinning_top::Spinlock,
};

static WAKER: AtomicWaker = AtomicWaker::new();

pub async fn task() {
    let registers = Spinlock::new(iter_devices().next().unwrap());
    let mut xhci = Xhci::new(&registers);
    let mut event_ring = event::Ring::new(&registers);
    let mut command_ring = command::Ring::new(&registers);
    let _dcbaa = DeviceContextBaseAddressArray::new(&registers);
    xhci.init();

    command_ring.send_noop();

    while let Some(trb) = event_ring.next().await {
        info!("TRB: {:?}", trb);
    }
}

pub struct Xhci<'a> {
    registers: &'a Spinlock<Registers>,
}

impl<'a> Xhci<'a> {
    fn init(&mut self) {
        self.get_ownership_from_bios();
        self.reset();
        self.wait_until_ready();
        self.set_num_of_enabled_slots();
        self.run();
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

    fn run(&mut self) {
        self.registers.lock().run_hc()
    }

    fn new(registers: &'a Spinlock<Registers>) -> Self {
        Self { registers }
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
