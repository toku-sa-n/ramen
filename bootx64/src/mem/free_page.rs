// SPDX-License-Identifier: GPL-3.0-or-later

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
            .allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 1)
            .expect_success("Failed to allocate memory for free memory"),
    )
}
