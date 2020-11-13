// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::Registers, crate::mem::allocator::page_box::PageBox, alloc::rc::Rc, core::cell::RefCell,
    x86_64::PhysAddr,
};

pub struct DeviceContextBaseAddressArray {
    arr: PageBox<[usize]>,
    registers: Rc<RefCell<Registers>>,
}
impl<'a> DeviceContextBaseAddressArray {
    pub fn new(registers: Rc<RefCell<Registers>>) -> Self {
        let arr = PageBox::new_slice(0, Self::num_of_slots(&registers));
        Self { arr, registers }
    }

    pub fn init(&self) {
        self.register_address_to_xhci_register();
    }

    fn num_of_slots(registers: &Rc<RefCell<Registers>>) -> usize {
        let params1 = &registers.borrow().hc_capability.hcs_params_1;
        (params1.read().max_slots() + 1).into()
    }

    fn register_address_to_xhci_register(&self) {
        let dcbaap = &mut self.registers.borrow_mut().hc_operational.dcbaap;
        dcbaap.update(|dcbaap| dcbaap.set(self.phys_addr()));
    }

    fn phys_addr(&self) -> PhysAddr {
        self.arr.phys_addr()
    }
}
