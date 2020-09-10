// SPDX-License-Identifier: GPL-3.0-or-later

pub mod reserved;

use uefi::table::boot;

#[repr(C)]
pub struct Map {
    ptr: *mut boot::MemoryDescriptor,
    num_descriptors: usize,
}

impl Map {
    pub fn new(ptr: *mut boot::MemoryDescriptor, num_descriptors: usize) -> Self {
        Self {
            ptr,
            num_descriptors,
        }
    }
}
