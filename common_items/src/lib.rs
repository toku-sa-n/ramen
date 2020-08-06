#![no_std]

extern crate uefi;

use core::mem::size_of;
use core::ptr;
use uefi::proto::console::gop;
use uefi::table::boot;

const INIT_RSP: usize = 0xffff_ffff_800a_1000 - size_of::<BootInfo>();

pub struct VramInfo {
    bpp: u16,
    screen_x: u16,
    screen_y: u16,
    ptr: u64,
}

impl VramInfo {
    pub fn new_from_gop(gop: &mut gop::GraphicsOutput) -> Self {
        let (screen_x, screen_y) = gop.current_mode_info().resolution();

        Self {
            bpp: 32,
            screen_x: screen_x as u16,
            screen_y: screen_y as u16,
            ptr: gop.frame_buffer().as_mut_ptr() as u64,
        }
    }
}

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

    pub fn set(self) -> () {
        unsafe {
            ptr::write(INIT_RSP as *mut BootInfo, self);
        }
    }
}
