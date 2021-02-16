// SPDX-License-Identifier: GPL-3.0-or-later

use crate::device::pci::xhci::registers;

pub struct Resetter {
    port_number: u8,
}
impl Resetter {
    pub fn new(port_number: u8) -> Self {
        Self { port_number }
    }

    pub fn reset(&mut self) {
        self.start_resetting();
        self.wait_until_reset_is_completed();
    }

    fn start_resetting(&mut self) {
        registers::handle(|r| {
            r.port_register_set
                .update_at((self.port_number - 1).into(), |r| {
                    r.portsc.set_port_reset(true)
                })
        });
    }

    fn wait_until_reset_is_completed(&self) {
        registers::handle(|r| {
            while !r
                .port_register_set
                .read_at((self.port_number - 1).into())
                .portsc
                .port_reset_changed()
            {}
        });
    }
}
