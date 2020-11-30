// SPDX-License-Identifier: GPL-3.0-or-later

use crate::device::pci::xhci::structures::registers::Registers;

pub struct Resetter<'a> {
    registers: &'a mut Registers,
    slot: u8,
}
impl<'a> Resetter<'a> {
    pub fn new(registers: &'a mut Registers, slot: u8) -> Self {
        Self { registers, slot }
    }

    pub fn reset(&mut self) {
        self.start_resetting();
        self.wait_until_reset_is_completed();
    }

    fn start_resetting(&mut self) {
        let r = &mut self.registers.operational.port_registers;
        r.update((self.slot - 1).into(), |r| r.port_sc.set_port_reset(true))
    }

    fn wait_until_reset_is_completed(&self) {
        let r = &self.registers.operational.port_registers;
        while !r.read((self.slot - 1).into()).port_sc.port_reset_changed() {}
    }
}
