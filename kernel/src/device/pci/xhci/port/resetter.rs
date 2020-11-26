// SPDX-License-Identifier: GPL-3.0-or-later

use core::cell::RefCell;

use alloc::rc::Rc;

use crate::device::pci::xhci::structures::registers::Registers;

pub struct Resetter {
    registers: Rc<RefCell<Registers>>,
    slot: u8,
}
impl Resetter {
    pub fn new(registers: Rc<RefCell<Registers>>, slot: u8) -> Self {
        Self { registers, slot }
    }

    pub fn reset(&mut self) {
        self.start_resetting();
        self.wait_until_reset_is_completed();
    }

    fn start_resetting(&mut self) {
        let r = &mut self.registers.borrow_mut().operational.port_registers;
        r.update((self.slot - 1).into(), |r| r.port_sc.set_port_reset(true))
    }

    fn wait_until_reset_is_completed(&self) {
        let r = &self.registers.borrow().operational.port_registers;
        while !r.read((self.slot - 1).into()).port_sc.port_reset_changed() {}
    }
}
