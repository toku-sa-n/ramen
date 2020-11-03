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
        let arr = PageBox::new_slice(usize::from(registers.lock().num_of_device_slots()) + 1);
        let dcbaa = Self { arr, registers };
        dcbaa.init();
        dcbaa
    }

    fn init(&self) {
        self.registers.lock().set_dcbaap(self.phys_addr())
    }

    fn phys_addr(&self) -> PhysAddr {
        self.arr.phys_addr()
    }
}
