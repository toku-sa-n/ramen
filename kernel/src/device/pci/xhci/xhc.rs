// SPDX-License-Identifier: GPL-3.0-or-later

pub struct Xhc;
impl Xhc {
    pub fn new() -> Self {
        Xhc
    }

    pub fn init(&mut self) {
        self.get_ownership_from_bios();
        self.stop_and_reset();
        self.set_num_of_enabled_slots();
    }

    pub fn run(&mut self) {
        super::handle_registers(|r| {
            let o = &mut r.operational;
            o.usb_cmd.update(|o| o.set_run_stop(true));
            while o.usb_sts.read().hc_halted() {}
        });
    }

    fn get_ownership_from_bios(&mut self) {
        super::handle_registers(|r| {
            if let Some(ref mut leg_sup_cap) = r.usb_legacy_support_capability {
                let leg_sup = &mut leg_sup_cap.usb_leg_sup;
                leg_sup.update(|s| s.os_request_ownership(true));

                while leg_sup.read().bios_owns_hc() || !leg_sup.read().os_owns_hc() {}
            }
        })
    }

    fn stop_and_reset(&mut self) {
        self.stop();
        self.wait_until_halt();
        self.reset();
    }

    fn stop(&mut self) {
        super::handle_registers(|r| {
            let c = &mut r.operational.usb_cmd;
            c.update(|c| c.set_run_stop(false));
        })
    }

    fn wait_until_halt(&self) {
        super::handle_registers(|r| {
            let s = &r.operational.usb_sts;
            while !s.read().hc_halted() {}
        })
    }

    fn reset(&mut self) {
        self.start_resetting();
        self.wait_until_reset_completed();
        self.wait_until_ready();
    }

    fn start_resetting(&mut self) {
        super::handle_registers(|r| {
            let c = &mut r.operational.usb_cmd;
            c.update(|c| c.set_hc_reset(true));
        })
    }

    fn wait_until_reset_completed(&self) {
        super::handle_registers(|r| {
            let c = &r.operational.usb_cmd;
            while c.read().hc_reset() {}
        })
    }

    fn wait_until_ready(&self) {
        super::handle_registers(|r| {
            let s = &r.operational.usb_sts;
            while s.read().controller_not_ready() {}
        })
    }

    fn set_num_of_enabled_slots(&mut self) {
        let n = self.num_of_device_slots();
        super::handle_registers(|r| {
            let c = &mut r.operational.config;
            c.update(|c| c.set_max_device_slots_enabled(n))
        })
    }

    fn num_of_device_slots(&self) -> u8 {
        super::handle_registers(|r| {
            let p = &r.capability.hcs_params_1;
            p.read().max_slots()
        })
    }
}
