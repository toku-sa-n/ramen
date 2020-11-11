// SPDX-License-Identifier: GPL-3.0-or-later

mod dcbaa;
mod port;
mod register;
mod ring;

use {
    super::config::bar,
    alloc::rc::Rc,
    core::cell::RefCell,
    dcbaa::DeviceContextBaseAddressArray,
    futures_util::task::AtomicWaker,
    register::Registers,
    ring::{command, event},
};

static WAKER: AtomicWaker = AtomicWaker::new();

pub async fn task() {
    let registers = Rc::new(RefCell::new(iter_devices().next().unwrap()));
    let (_xhc, event_ring, mut command_ring, _dcbaa, mut ports) = init(&registers);
    command_ring.send_noop();

    ports.enable_all_connected_ports();

    event::task(event_ring).await;
}

fn init(
    registers: &Rc<RefCell<Registers>>,
) -> (
    Xhc,
    event::Ring,
    command::Ring,
    DeviceContextBaseAddressArray,
    port::Collection,
) {
    let mut xhc = Xhc::new(registers.clone());
    let mut event_ring = event::Ring::new(registers.clone());
    let mut command_ring = command::Ring::new(registers.clone());
    let dcbaa = DeviceContextBaseAddressArray::new(registers.clone());
    let ports = port::Collection::new(&registers);

    xhc.init();

    event_ring.init();
    command_ring.init();
    dcbaa.init();

    xhc.run();

    (xhc, event_ring, command_ring, dcbaa, ports)
}

pub struct Xhc {
    registers: Rc<RefCell<Registers>>,
}

impl Xhc {
    fn new(registers: Rc<RefCell<Registers>>) -> Self {
        Self { registers }
    }

    fn init(&mut self) {
        self.get_ownership_from_bios();
        self.stop_and_reset();
        self.set_num_of_enabled_slots();
    }

    fn get_ownership_from_bios(&mut self) {
        if let Some(ref mut usb_leg_sup_cap) =
            self.registers.borrow_mut().usb_legacy_support_capability
        {
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
        let usb_cmd = &mut self.registers.borrow_mut().hc_operational.usb_cmd;
        usb_cmd.update(|cmd| cmd.set_run_stop(false));
    }

    fn wait_until_halt(&self) {
        let usb_sts = &self.registers.borrow().hc_operational.usb_sts;
        while !usb_sts.read().hc_halted() {}
    }

    fn reset(&mut self) {
        self.start_resetting();
        self.wait_until_reset_completed();
        self.wait_until_ready();
    }

    fn start_resetting(&mut self) {
        let usb_cmd = &mut self.registers.borrow_mut().hc_operational.usb_cmd;
        usb_cmd.update(|cmd| cmd.set_hc_reset(true));
    }

    fn wait_until_reset_completed(&self) {
        let usb_cmd = &self.registers.borrow().hc_operational.usb_cmd;
        while usb_cmd.read().hc_reset() {}
    }

    fn wait_until_ready(&self) {
        let usb_sts = &self.registers.borrow().hc_operational.usb_sts;
        while usb_sts.read().controller_not_ready() {}
    }

    fn set_num_of_enabled_slots(&mut self) {
        let num_of_device_slots = self.num_of_device_slots();
        let config = &mut self.registers.borrow_mut().hc_operational.config;
        config.update(|config| config.set_max_device_slots_enabled(num_of_device_slots))
    }

    fn num_of_device_slots(&self) -> u8 {
        let params1 = &self.registers.borrow().hc_capability.hcs_params_1;
        params1.read().max_slots()
    }

    fn run(&mut self) {
        let operational = &mut self.registers.borrow_mut().hc_operational;
        operational.usb_cmd.update(|oper| oper.set_run_stop(true));
        while operational.usb_sts.read().hc_halted() {}
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
