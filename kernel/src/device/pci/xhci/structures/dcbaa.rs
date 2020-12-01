// SPDX-License-Identifier: GPL-3.0-or-later

use super::registers::Registers;
use crate::mem::allocator::page_box::PageBox;
use alloc::rc::Rc;
use core::{
    cell::RefCell,
    ops::{Index, IndexMut},
};
use x86_64::PhysAddr;

pub struct DeviceContextBaseAddressArray {
    arr: PageBox<[PhysAddr]>,
    registers: Rc<RefCell<Registers>>,
}
impl<'a> DeviceContextBaseAddressArray {
    pub fn new(registers: Rc<RefCell<Registers>>) -> Self {
        let arr = PageBox::new_slice(PhysAddr::zero(), Self::num_of_slots(&registers.borrow()));
        Self { arr, registers }
    }

    pub fn init(&self) {
        self.register_address_to_xhci_register();
    }

    fn num_of_slots(registers: &Registers) -> usize {
        let params1 = &registers.capability.hcs_params_1;
        (params1.read().max_slots() + 1).into()
    }

    fn register_address_to_xhci_register(&self) {
        let dcbaap = &mut self.registers.borrow_mut().operational.dcbaap;
        dcbaap.update(|dcbaap| dcbaap.set(self.phys_addr()));
    }

    fn phys_addr(&self) -> PhysAddr {
        self.arr.phys_addr()
    }
}
impl Index<usize> for DeviceContextBaseAddressArray {
    type Output = PhysAddr;
    fn index(&self, index: usize) -> &Self::Output {
        &self.arr[index]
    }
}
impl IndexMut<usize> for DeviceContextBaseAddressArray {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.arr[index]
    }
}
