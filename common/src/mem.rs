// SPDX-License-Identifier: GPL-3.0-or-later

pub mod reserved;

use core::ptr::NonNull;
use uefi::table::boot;

#[repr(C)]
pub struct Map {
    ptr: NonNull<boot::MemoryDescriptor>,
    num_descriptors: usize,
}

impl Map {
    pub fn new(ptr: NonNull<boot::MemoryDescriptor>, num_descriptors: usize) -> Self {
        Self {
            ptr,
            num_descriptors,
        }
    }
}
