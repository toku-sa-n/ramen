// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![feature(const_fn)]

pub mod constant;
pub mod debug;
pub mod mem;
pub mod size;
pub mod vram;

extern crate uefi;
extern crate x86_64;

use crate::constant::INIT_RSP;
use core::ptr;
use size::{Byte, Size};

#[repr(C)]
pub struct BootInfo {
    vram_info: vram::Info,
    mem_map: mem::Map,
}

impl BootInfo {
    pub fn new(vram_info: vram::Info, mem_map: mem::Map) -> Self {
        Self { vram_info, mem_map }
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
