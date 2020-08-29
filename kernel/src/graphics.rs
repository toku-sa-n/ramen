// SPDX-License-Identifier: GPL-3.0-or-later

pub mod font;

#[macro_use]
pub mod screen;

use crate::common;
use crate::common::constant::VRAM_ADDR;
use common::boot;
use core::ptr;

// Copy trait is needed for constructing MouseCursor struct
// If you are unsure, remove Copy trait from this struct and see the error messages.
#[derive(Clone, Copy)]
pub struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

impl RGB {
    pub fn new(hex: u32) -> Self {
        Self {
            r: ((hex & 0xFF0000) >> 16) as u8,
            g: ((hex & 0x00FF00) >> 8) as u8,
            b: (hex & 0x0000FF) as u8,
        }
    }
}

#[derive(Clone)]
pub struct Vram {
    pub bits_per_pixel: usize,
    pub x_len: usize,
    pub y_len: usize,
    pub ptr: *mut u8,
}

impl Vram {
    pub fn new_from_boot_info(boot_info: &boot::Info) -> Self {
        let (x_len, y_len) = boot_info.vram().resolution();

        Self {
            bits_per_pixel: boot_info.vram().bpp(),
            x_len,
            y_len,
            ptr: VRAM_ADDR.as_mut_ptr(),
        }
    }
}

impl Vram {
    pub unsafe fn set_color(&self, coord: screen::Coord<isize>, rgb: RGB) -> () {
        let base_ptr: *mut u8 = self
            .ptr
            .offset((coord.y * self.x_len as isize + coord.x) * self.bits_per_pixel as isize / 8);

        // The order of `RGB` is right.
        // See: https://wiki.osdev.org/Drawing_In_Protected_Mode
        ptr::write(base_ptr.offset(0), rgb.b);
        ptr::write(base_ptr.offset(1), rgb.g);
        ptr::write(base_ptr.offset(2), rgb.r);
    }
}
