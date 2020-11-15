// SPDX-License-Identifier: GPL-3.0-or-later

use {super::registers::Registers, alloc::rc::Rc, core::cell::RefCell};

pub struct Ahc {
    registers: Rc<RefCell<Registers>>,
}
impl Ahc {
    pub fn new(registers: Rc<RefCell<Registers>>) -> Self {
        Self { registers }
    }

    pub fn indicate_system_software_is_ahci_aware(&mut self) {
        let ghc = &mut self.registers.borrow_mut().generic.ghc;
        ghc.update(|ghc| ghc.set_ahci_enable(true));
    }

    pub fn reset(&mut self) {
        self.start_resetting();
        self.wait_until_reset_is_completed();
    }

    pub fn get_ownership_from_bios(&mut self) {
        self.request_ownership_to_bios();
        self.wait_until_ownership_is_moved();
    }

    fn start_resetting(&mut self) {
        let registers = &mut self.registers.borrow_mut();
        let ghc = &mut registers.generic.ghc;
        ghc.update(|ghc| ghc.set_hba_reset(true));
    }

    fn wait_until_reset_is_completed(&self) {
        let registers = &self.registers.borrow();
        let ghc = &registers.generic.ghc;
        while ghc.read().hba_reset() {}
    }

    fn request_ownership_to_bios(&mut self) {
        let registers = &mut self.registers.borrow_mut();
        let bohc = &mut registers.generic.bohc;
        bohc.update(|bohc| bohc.set_os_owned_semaphore(true));
    }

    fn wait_until_ownership_is_moved(&self) {
        let registers = &self.registers.borrow();
        let bohc = &registers.generic.bohc;
        while bohc.read().os_owned_semaphore() && !bohc.read().bios_owned_semaphore() {}
    }
}
