// SPDX-License-Identifier: GPL-3.0-or-later

pub mod font;

#[macro_use]
pub mod screen;

use common::{constant::VRAM_ADDR, kernelboot};
use conquer_once::spin::{Lazy, OnceCell};
use core::{convert::TryFrom, ptr};
use vek::Vec2;
use x86_64::VirtAddr;

static VRAM: Lazy<OnceCell<Vram>> = Lazy::new(OnceCell::uninit);

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
            r: u8::try_from((hex & 0x00FF_0000) >> 16).unwrap(),
            g: u8::try_from((hex & 0x0000_FF00) >> 8).unwrap(),
            b: u8::try_from(hex & 0x0000_00FF).unwrap(),
        }
    }
}

#[derive(Clone)]
pub struct Vram {
    bits_per_pixel: usize,
    resolution: Vec2<usize>,
    ptr: VirtAddr,
}

impl Vram {
    pub fn init(boot_info: &kernelboot::Info) {
        VRAM.try_init_once(|| Self::new_from_boot_info(boot_info))
            .unwrap();
    }

    fn new_from_boot_info(boot_info: &kernelboot::Info) -> Self {
        let vram = boot_info.vram();

        let (x_len, y_len) = vram.resolution();
        let resolution = Vec2::new(x_len, y_len);

        Self::new(vram.bpp(), resolution, VRAM_ADDR)
    }

    fn new(bits_per_pixel: usize, resolution: Vec2<usize>, ptr: VirtAddr) -> Self {
        Self {
            bits_per_pixel,
            resolution,
            ptr,
        }
    }

    fn get() -> &'static Vram {
        VRAM.try_get().expect("VRAM not initialized")
    }

    pub fn resolution() -> &'static Vec2<usize> {
        &Vram::get().resolution
    }

    pub unsafe fn set_color(coord: &Vec2<isize>, rgb: RGB) {
        let vram = Self::get();

        let base_ptr = (usize::try_from(vram.ptr.as_u64()).unwrap()
            + (usize::try_from(coord.y).unwrap() * Vram::resolution().x
                + usize::try_from(coord.x).unwrap())
                * vram.bits_per_pixel
                / 8) as *mut u8;

        // The order of `RGB` is right.
        // See: https://wiki.osdev.org/Drawing_In_Protected_Mode
        ptr::write(base_ptr.offset(0), rgb.b);
        ptr::write(base_ptr.offset(1), rgb.g);
        ptr::write(base_ptr.offset(2), rgb.r);
    }
}
