// SPDX-License-Identifier: GPL-3.0-or-later

use crate::constant::INIT_RSP;
use crate::mem;
use crate::vram;
use core::ptr;

#[repr(C)]
pub struct Info {
    vram_info: vram::Info,
    mem_map: mem::Map,
}

impl Info {
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
