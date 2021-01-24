// SPDX-License-Identifier: GPL-3.0-or-later

use crate::device::pci::xhci;

pub struct Resetter {
    slot: u8,
}
impl Resetter {
    pub fn new(slot: u8) -> Self {
        Self { slot }
    }

    pub fn reset(&mut self) {
        self.start_resetting();
        self.wait_until_reset_is_completed();
    }

    fn start_resetting(&mut self) {
        xhci::handle_registers(|r| {
            let p = &mut r.operational.port_registers;
            p.update_at((self.slot - 1).into(), |r| r.port_sc.set_port_reset(true))
        });
    }

    fn wait_until_reset_is_completed(&self) {
        xhci::handle_registers(|r| {
            let p = &r.operational.port_registers;
            while !p
                .read_at((self.slot - 1).into())
                .port_sc
                .port_reset_changed()
            {}
        });
    }
}
