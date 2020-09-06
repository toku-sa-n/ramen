// SPDX-License-Identifier: GPL-3.0-or-later

use common::constant::NUM_OF_PAGES_STACK;
use uefi::table::boot;
use uefi::table::boot::{AllocateType, MemoryType};
use uefi::ResultExt;
use x86_64::PhysAddr;

pub fn allocate(boot_services: &boot::BootServices) -> PhysAddr {
    PhysAddr::new(
        boot_services
            .allocate_pages(
                AllocateType::AnyPages,
                MemoryType::LOADER_DATA,
                NUM_OF_PAGES_STACK.as_bytes().as_usize(),
            )
            .expect_success("Failed to allocate memory for the stack"),
    )
}
