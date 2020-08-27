#![no_std]
#![feature(const_fn)]

pub mod constant;
pub mod debug;
pub mod size;

extern crate uefi;
extern crate x86_64;

use crate::constant::INIT_RSP;
use core::ptr;
use size::{Byte, Size};
use uefi::proto::console::gop;
use uefi::table::boot;
use x86_64::addr::PhysAddr;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct VramInfo {
    bpp: usize,
    screen_x: usize,
    screen_y: usize,
    ptr: PhysAddr,
}

impl VramInfo {
    pub fn new_from_gop(gop: &mut gop::GraphicsOutput) -> Self {
        let (screen_x, screen_y) = gop.current_mode_info().resolution();

        Self {
            bpp: 32,
            screen_x: screen_x,
            screen_y: screen_y,
            ptr: PhysAddr::new(gop.frame_buffer().as_mut_ptr() as u64),
        }
    }

    pub fn bpp(&self) -> usize {
        self.bpp
    }

    pub fn resolution(&self) -> (usize, usize) {
        (self.screen_x, self.screen_y)
    }

    pub fn phys_ptr(&self) -> PhysAddr {
        self.ptr
    }

    pub fn bytes(&self) -> Size<Byte> {
        Size::new(self.screen_x as usize * self.screen_y as usize * self.bpp as usize / 8)
    }
}

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
    vram_info: VramInfo,
    mem_map_info: MemMapInfo,
}

impl BootInfo {
    pub fn new(vram_info: VramInfo, mem_map_info: MemMapInfo) -> Self {
        Self {
            vram_info,
            mem_map_info,
        }
    }

    pub fn vram(&self) -> VramInfo {
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
