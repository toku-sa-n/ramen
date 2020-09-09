// SPDX-License-Identifier: GPL-3.0-or-later

use common::constant::BYTES_KERNEL_HEAP;
use uefi::{
    table::{
        boot,
        boot::{AllocateType, MemoryType},
    },
    ResultExt,
};
use x86_64::PhysAddr;

pub fn allocate(boot_services: &boot::BootServices) -> PhysAddr {
    PhysAddr::new(
        boot_services
            .allocate_pages(
                AllocateType::AnyPages,
                MemoryType::LOADER_DATA,
                BYTES_KERNEL_HEAP.as_usize(),
            )
            .expect_success("Failed to allocate memory for kernel heap"),
    )
}
