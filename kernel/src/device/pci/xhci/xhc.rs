// SPDX-License-Identifier: GPL-3.0-or-later

pub fn init() {
    get_ownership_from_bios();
    stop_and_reset();
    set_num_of_enabled_slots();
}

pub fn run() {
    super::handle_registers(|r| {
        let o = &mut r.operational;
        o.usb_cmd.update(|o| o.set_run_stop(true));
        while o.usb_sts.read().hc_halted() {}
    });
}

pub fn ensure_no_error_occurs() {
    super::handle_registers(|r| {
        let s = r.operational.usb_sts.read();
        assert!(!s.hc_halted(), "HC is halted.");
        assert!(
            !s.host_system_error(),
            "An error occured on the host system."
        );
        assert!(!s.hc_error(), "An error occured on the xHC.");
    });
}

fn get_ownership_from_bios() {
    super::handle_registers(|r| {
        if let Some(ref mut leg_sup_cap) = r.usb_legacy_support_capability {
            let leg_sup = &mut leg_sup_cap.usb_leg_sup;
            leg_sup.update(|s| s.os_request_ownership(true));

            while leg_sup.read().bios_owns_hc() || !leg_sup.read().os_owns_hc() {}
        }
    })
}

fn stop_and_reset() {
    stop();
    wait_until_halt();
    reset();
}

fn stop() {
    super::handle_registers(|r| {
        let c = &mut r.operational.usb_cmd;
        c.update(|c| c.set_run_stop(false));
    })
}

fn wait_until_halt() {
    super::handle_registers(|r| {
        let s = &r.operational.usb_sts;
        while !s.read().hc_halted() {}
    })
}

fn reset() {
    start_resetting();
    wait_until_reset_completed();
    wait_until_ready();
}

fn start_resetting() {
    super::handle_registers(|r| {
        let c = &mut r.operational.usb_cmd;
        c.update(|c| c.set_hc_reset(true));
    })
}

fn wait_until_reset_completed() {
    super::handle_registers(|r| {
        let c = &r.operational.usb_cmd;
        while c.read().hc_reset() {}
    })
}

fn wait_until_ready() {
    super::handle_registers(|r| {
        let s = &r.operational.usb_sts;
        while s.read().controller_not_ready() {}
    })
}

fn set_num_of_enabled_slots() {
    let n = num_of_device_slots();
    super::handle_registers(|r| {
        let c = &mut r.operational.config;
        c.update(|c| c.set_max_device_slots_enabled(n))
    })
}

fn num_of_device_slots() -> u8 {
    super::handle_registers(|r| {
        let p = &r.capability.hcs_params_1;
        p.read().max_slots()
    })
}
