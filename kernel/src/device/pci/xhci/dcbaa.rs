// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::Registers, crate::mem::allocator::page_box::PageBox, spinning_top::Spinlock,
    x86_64::PhysAddr,
};

pub struct DeviceContextBaseAddressArray<'a> {
    arr: PageBox<[usize]>,
    registers: &'a Spinlock<Registers>,
}
impl<'a> DeviceContextBaseAddressArray<'a> {
    pub fn new(registers: &'a Spinlock<Registers>) -> Self {
        let arr = PageBox::new_slice(Self::num_of_slots(registers));
        Self { arr, registers }
    }

    pub fn init(&self) {
        self.register_address_to_xhci_register();
    }

    fn num_of_slots(registers: &'a Spinlock<Registers>) -> usize {
        (registers.lock().hc_capability.hcs_params_1.max_slots() + 1).into()
    }

    fn register_address_to_xhci_register(&self) {
        self.registers
            .lock()
            .hc_operational
            .dcbaap
            .set(self.phys_addr());
    }

    fn phys_addr(&self) -> PhysAddr {
        self.arr.phys_addr()
    }
}
