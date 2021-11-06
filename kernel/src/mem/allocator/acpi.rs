// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::mem,
    acpi::{AcpiHandler, PhysicalMapping},
    core::{convert::TryInto, ptr::NonNull},
    os_units::Bytes,
    x86_64::{PhysAddr, VirtAddr},
};

#[derive(Clone)]
pub(crate) struct Mapper;
impl AcpiHandler for Mapper {
    unsafe fn map_physical_region<U>(&self, p_addr: usize, sz: usize) -> PhysicalMapping<Self, U> {
        let p = PhysAddr::new(p_addr.try_into().unwrap());
        let bytes = Bytes::new(sz);

        // To call this method in the kernel mode, which is necessary to access APIC registers,
        // call `crate::mem::map_pages`, not the system call one.
        let virt = mem::map_pages(p, bytes);

        PhysicalMapping::new(
            p_addr,
            NonNull::new(virt.as_mut_ptr()).unwrap(),
            sz,
            sz,
            Self,
        )
    }

    fn unmap_physical_region<T>(region: &PhysicalMapping<Self, T>) {
        let virt = VirtAddr::new(region.virtual_start().as_ptr() as u64);
        let bytes = Bytes::new(region.region_length());
        mem::unmap_pages(virt, bytes);
    }
}
