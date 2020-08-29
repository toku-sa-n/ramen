// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![feature(const_fn)]

pub mod constant;
pub mod debug;
pub mod size;
pub mod vram;

extern crate uefi;
extern crate x86_64;

use crate::constant::INIT_RSP;
use core::ptr;
use size::{Byte, Size};
use uefi::table::boot;

#[repr(C)]
pub struct MemMapInfo {
    ptr: *mut boot::MemoryDescriptor,
    num_descriptors: usize,
}

impl MemMapInfo {
    pub fn new_from_slice(map: &mut [boot::MemoryDescriptor]) -> Self {
        let ptr = map.as_mut_ptr();
        let num_descriptors = map.len();

        Self {
            ptr,
            num_descriptors,
        }
    }
}

#[repr(C)]
pub struct BootInfo {
    vram_info: vram::Info,
    mem_map_info: MemMapInfo,
}

impl BootInfo {
    pub fn new(vram_info: vram::Info, mem_map_info: MemMapInfo) -> Self {
        Self {
            vram_info,
            mem_map_info,
        }
    }

    pub fn vram(&self) -> vram::Info {
        self.vram_info
    }

    pub fn set(self) -> () {
        unsafe {
            ptr::write(INIT_RSP.as_mut_ptr() as _, self);
        }
    }

    pub fn get() -> Self {
        unsafe { ptr::read(INIT_RSP.as_mut_ptr() as _) }
    }
}
