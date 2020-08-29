// SPDX-License-Identifier: GPL-3.0-or-later

pub mod reserved;

use uefi::table::boot;

#[repr(C)]
pub struct Map {
    ptr: *mut boot::MemoryDescriptor,
    num_descriptors: usize,
}

impl Map {
    pub fn new_from_slice(map: &mut [boot::MemoryDescriptor]) -> Self {
        let ptr = map.as_mut_ptr();
        let num_descriptors = map.len();

        Self {
            ptr,
            num_descriptors,
        }
    }
}
