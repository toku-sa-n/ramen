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
    let mut xhc = Xhc::new(&registers);
    let mut event_ring = event::Ring::new(&registers);
    let mut command_ring = command::Ring::new(&registers);
    let dcbaa = DeviceContextBaseAddressArray::new(&registers);

    xhc.init();

    event_ring.init();
    command_ring.init();
    dcbaa.init();

    xhc.run();

    command_ring.send_noop();

    xhc.check_connections_of_each_port();
    while let Some(trb) = event_ring.next().await {
        info!("TRB: {:?}", trb);
    }
}

pub struct Xhc<'a> {
    registers: &'a Spinlock<Registers>,
}

impl<'a> Xhc<'a> {
    fn new(registers: &'a Spinlock<Registers>) -> Self {
        Self { registers }
    }

    fn init(&mut self) {
        self.get_ownership_from_bios();
        self.stop_and_reset();
        self.set_num_of_enabled_slots();
    }

    fn get_ownership_from_bios(&mut self) {
        if let Some(ref mut usb_leg_sup_cap) = self.registers.lock().usb_legacy_support_capability {
            let usb_leg_sup = &mut usb_leg_sup_cap.usb_leg_sup;
            usb_leg_sup.update(|sup| sup.os_request_ownership(true));

            while usb_leg_sup.read().bios_owns_hc() || !usb_leg_sup.read().os_owns_hc() {}
        }
    }

    fn stop_and_reset(&mut self) {
        self.stop();
        self.wait_until_halt();
        self.reset();
    }

    fn stop(&mut self) {
        let usb_cmd = &mut self.registers.lock().hc_operational.usb_cmd;
        usb_cmd.update(|cmd| cmd.set_run_stop(false));
    }

    fn wait_until_halt(&self) {
        let usb_sts = &self.registers.lock().hc_operational.usb_sts;
        while !usb_sts.read().hc_halted() {}
    }

    fn reset(&mut self) {
        self.start_resetting();
        self.wait_until_reset_completed();
        self.wait_until_ready();
    }

    fn start_resetting(&mut self) {
        let usb_cmd = &mut self.registers.lock().hc_operational.usb_cmd;
        usb_cmd.update(|cmd| cmd.set_hc_reset(true));
    }

    fn wait_until_reset_completed(&self) {
        let usb_cmd = &self.registers.lock().hc_operational.usb_cmd;
        while usb_cmd.read().hc_reset() {}
    }

    fn wait_until_ready(&self) {
        let usb_sts = &self.registers.lock().hc_operational.usb_sts;
        while usb_sts.read().controller_not_ready() {}
    }

    fn set_num_of_enabled_slots(&mut self) {
        let num_of_device_slots = self.num_of_device_slots();
        let config = &mut self.registers.lock().hc_operational.config;
        config.update(|config| config.set_max_device_slots_enabled(num_of_device_slots))
    }

    fn num_of_device_slots(&self) -> u8 {
        let params1 = &self.registers.lock().hc_capability.hcs_params_1;
        params1.read().max_slots()
    }

    fn run(&mut self) {
        let operational = &mut self.registers.lock().hc_operational;
        operational.usb_cmd.update(|oper| oper.set_run_stop(true));
        while operational.usb_sts.read().hc_halted() {}
    }

    fn check_connections_of_each_port(&self) {
        for i in 0..self.num_of_ports() {
            self.print_port_status(i);
        }
    }

    fn num_of_ports(&self) -> usize {
        self.registers
            .lock()
            .hc_capability
            .hcs_params_1
            .read()
            .max_ports()
            .into()
    }

    fn print_port_status(&self, index: usize) {
        info!(
            "Port {}: {}",
            index,
            self.registers
                .lock()
                .hc_operational
                .port_registers
                .read(index)
                .port_sc
                .current_connect_status()
        );
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
