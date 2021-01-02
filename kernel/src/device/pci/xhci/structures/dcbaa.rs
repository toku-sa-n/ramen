// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{device::pci::xhci, mem::allocator::page_box::PageBox};
use conquer_once::spin::Lazy;
use core::ops::{Index, IndexMut};
use spinning_top::Spinlock;
use x86_64::PhysAddr;

static DCBAA: Lazy<Spinlock<DeviceContextBaseAddressArray>> =
    Lazy::new(|| Spinlock::new(DeviceContextBaseAddressArray::new()));

pub fn init() {
    DCBAA.lock().init();
}

pub fn register(port_id: usize, a: PhysAddr) {
    DCBAA.lock()[port_id] = a;
}

pub struct DeviceContextBaseAddressArray {
    arr: PageBox<[PhysAddr]>,
}
impl<'a> DeviceContextBaseAddressArray {
    fn new() -> Self {
        let arr = PageBox::user_slice(PhysAddr::zero(), Self::num_of_slots());
        Self { arr }
    }

    fn init(&self) {
        self.register_address_to_xhci_register();
    }

    fn num_of_slots() -> usize {
        xhci::handle_registers(|r| {
            let p = &r.capability.hcs_params_1;
            (p.read().max_slots() + 1).into()
        })
    }

    fn register_address_to_xhci_register(&self) {
        xhci::handle_registers(|r| {
            let p = &mut r.operational.dcbaap;
            p.update(|dcbaap| dcbaap.set(self.phys_addr()));
        })
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
