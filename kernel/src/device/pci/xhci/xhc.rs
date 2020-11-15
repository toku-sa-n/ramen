// SPDX-License-Identifier: GPL-3.0-or-later

use {super::register::Registers, alloc::rc::Rc, core::cell::RefCell};

pub struct Xhc {
    registers: Rc<RefCell<Registers>>,
}

impl Xhc {
    pub fn new(registers: Rc<RefCell<Registers>>) -> Self {
        Self { registers }
    }

    pub fn init(&mut self) {
        self.get_ownership_from_bios();
        self.stop_and_reset();
        self.set_num_of_enabled_slots();
    }

    pub fn run(&mut self) {
        let operational = &mut self.registers.borrow_mut().hc_operational;
        operational.usb_cmd.update(|oper| oper.set_run_stop(true));
        while operational.usb_sts.read().hc_halted() {}
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
}
