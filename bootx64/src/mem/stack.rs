// SPDX-License-Identifier: GPL-3.0-or-later

use {
    predefined_mmap::NUM_OF_PAGES_STACK,
    uefi::{
        table::{
            boot,
            boot::{AllocateType, MemoryType},
        },
        ResultExt,
    },
    x86_64::PhysAddr,
};

#[must_use]
pub fn allocate(bs: &boot::BootServices) -> PhysAddr {
    PhysAddr::new(
        bs.allocate_pages(
            AllocateType::AnyPages,
            MemoryType::LOADER_DATA,
            NUM_OF_PAGES_STACK.as_bytes().as_usize(),
        )
        .expect_success("Failed to allocate memory for the stack"),
    )
}
