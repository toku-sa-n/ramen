// SPDX-License-Identifier: GPL-3.0-or-later

use super::structures::registers;
use xhci::extended_capabilities::ExtendedCapability;

pub fn init() {
    get_ownership_from_bios();
    stop_and_reset();
    set_num_of_enabled_slots();
}

pub fn run() {
    registers::handle(|r| {
        let o = &mut r.operational;
        o.usbcmd.update(|u| u.set_run_stop(true));
        while o.usbsts.read().hc_halted() {}
    });
}

pub fn ensure_no_error_occurs() {
    registers::handle(|r| {
        let s = r.operational.usbsts.read();
        assert!(!s.hc_halted(), "HC is halted.");
        assert!(
            !s.host_system_error(),
            "An error occured on the host system."
        );
        assert!(!s.host_controller_error(), "An error occured on the xHC.");
    });
}

fn get_ownership_from_bios() {
    if let Some(iter) = super::iter_extended_capabilities() {
        for c in iter.filter_map(Result::ok) {
            if let ExtendedCapability::UsbLegacySupportCapability(mut l) = c {
                l.update(|s| s.set_hc_os_owned_semaphore(true));

                while l.read().hc_bios_owned_semaphore() || !l.read().hc_os_owned_semaphore() {}
            }
        }
    }
}

fn stop_and_reset() {
    stop();
    wait_until_halt();
    reset();
}

fn stop() {
    registers::handle(|r| {
        r.operational.usbcmd.update(|u| u.set_run_stop(false));
    })
}

fn wait_until_halt() {
    registers::handle(|r| while !r.operational.usbsts.read().hc_halted() {})
}

fn reset() {
    start_resetting();
    wait_until_reset_completed();
    wait_until_ready();
}

fn start_resetting() {
    registers::handle(|r| {
        r.operational
            .usbcmd
            .update(|u| u.set_host_controller_reset(true))
    })
}

fn wait_until_reset_completed() {
    registers::handle(
        |r| {
            while r.operational.usbcmd.read().host_controller_reset() {}
        },
    )
}

fn wait_until_ready() {
    registers::handle(
        |r| {
            while r.operational.usbsts.read().controller_not_ready() {}
        },
    )
}

fn set_num_of_enabled_slots() {
    let n = num_of_device_slots();
    registers::handle(|r| {
        r.operational
            .config
            .update(|c| c.set_max_device_slots_enabled(n));
    })
}

fn num_of_device_slots() -> u8 {
    registers::handle(|r| r.capability.hcsparams1.read().number_of_device_slots())
}
