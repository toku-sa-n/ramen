// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::registers,
    conquer_once::spin::Lazy,
    core::ops::{Index, IndexMut},
    page_box::PageBox,
    spinning_top::Spinlock,
    x86_64::PhysAddr,
};

static DCBAA: Lazy<Spinlock<DeviceContextBaseAddressArray>> =
    Lazy::new(|| Spinlock::new(DeviceContextBaseAddressArray::new()));

pub(crate) fn init() {
    DCBAA.lock().init();
}

pub(crate) fn register(port_id: usize, a: PhysAddr) {
    DCBAA.lock()[port_id] = a;
}

pub(crate) struct DeviceContextBaseAddressArray {
    arr: PageBox<[PhysAddr]>,
}
impl DeviceContextBaseAddressArray {
    fn new() -> Self {
        let arr = PageBox::new_slice(PhysAddr::zero(), Self::num_of_slots());
        Self { arr }
    }

    fn init(&self) {
        self.register_address_to_xhci_register();
    }

    fn num_of_slots() -> usize {
        registers::handle(|r| {
            r.capability
                .hcsparams1
                .read_volatile()
                .number_of_device_slots()
                + 1
        })
        .into()
    }

    fn register_address_to_xhci_register(&self) {
        registers::handle(|r| {
            let _ = &self;
            r.operational.dcbaap.update_volatile(|d| {
                let _ = &self;
                d.set(self.phys_addr().as_u64());
            });
        });
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
