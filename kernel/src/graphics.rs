// SPDX-License-Identifier: GPL-3.0-or-later

pub mod font;

#[macro_use]
pub mod screen;

use crate::common;
use common::boot;
use common::constant::VRAM_ADDR;
use core::ptr;
use lazy_static::lazy_static;
use x86_64::VirtAddr;

lazy_static! {
    pub static ref VRAM: Vram = {
        let boot_info = boot::Info::get();
        let (x_len, y_len) = boot_info.vram().resolution();

        Vram::new(boot_info.vram().bpp(), x_len, y_len, VRAM_ADDR)
    };
}

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
    bits_per_pixel: usize,
    x_len: usize,
    y_len: usize,
    ptr: VirtAddr,
}

impl Vram {
    fn new(bits_per_pixel: usize, x_len: usize, y_len: usize, ptr: VirtAddr) -> Self {
        Self {
            bits_per_pixel,
            x_len,
            y_len,
            ptr,
        }
    }

    pub fn x_len(&self) -> usize {
        self.x_len
    }

    pub fn y_len(&self) -> usize {
        self.y_len
    }

    pub fn ptr(&self) -> VirtAddr {
        self.ptr
    }
}

impl Vram {
    pub unsafe fn set_color(&self, coord: screen::Coord<isize>, rgb: RGB) -> () {
        let base_ptr: *mut u8 = (self.ptr.as_mut_ptr() as *mut u8)
            .offset((coord.y * self.x_len as isize + coord.x) * self.bits_per_pixel as isize / 8);

        // The order of `RGB` is right.
        // See: https://wiki.osdev.org/Drawing_In_Protected_Mode
        ptr::write(base_ptr.offset(0), rgb.b);
        ptr::write(base_ptr.offset(1), rgb.g);
        ptr::write(base_ptr.offset(2), rgb.r);
    }
}
