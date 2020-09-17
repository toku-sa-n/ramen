// SPDX-License-Identifier: GPL-3.0-or-later

pub mod font;

#[macro_use]
pub mod screen;

use common::{constant::VRAM_ADDR, kernelboot};
use conquer_once::spin::{Lazy, OnceCell};
use core::{convert::TryFrom, fmt, ptr};
use rgb::RGB8;
use vek::Vec2;
use x86_64::VirtAddr;

static VRAM: Lazy<OnceCell<Vram>> = Lazy::new(OnceCell::uninit);

#[derive(Clone)]
pub struct Vram {
    bits_per_pixel: i32,
    resolution: Vec2<i32>,
    ptr: VirtAddr,
}

impl Vram {
    pub fn init(boot_info: &kernelboot::Info) {
        VRAM.try_init_once(|| Self::new_from_boot_info(boot_info))
            .unwrap();
    }

    pub fn resolution() -> &'static Vec2<i32> {
        &Vram::get().resolution
    }

    pub fn display() -> impl core::fmt::Display {
        Self::get()
    }

    pub fn bpp() -> i32 {
        Vram::get().bits_per_pixel
    }

    pub fn ptr() -> VirtAddr {
        Vram::get().ptr
    }

    pub unsafe fn set_color(coord: Vec2<i32>, rgb: RGB8) {
        let vram = Self::get();

        let offset_from_base = (coord.y * Vram::resolution().x + coord.x) * vram.bits_per_pixel / 8;

        let ptr = vram.ptr.as_mut_ptr::<u8>().offset(offset_from_base as _);

        // The order of `RGB` is right.
        // See: https://wiki.osdev.org/Drawing_In_Protected_Mode
        ptr::write(ptr.offset(0), rgb.b);
        ptr::write(ptr.offset(1), rgb.g);
        ptr::write(ptr.offset(2), rgb.r);
    }
    fn new_from_boot_info(boot_info: &kernelboot::Info) -> Self {
        let vram = boot_info.vram();

        let (x_len, y_len) = vram.resolution();
        let resolution = Vec2::new(x_len, y_len);

        Self::new(vram.bpp(), resolution, VRAM_ADDR)
    }

    fn new(bits_per_pixel: i32, resolution: Vec2<i32>, ptr: VirtAddr) -> Self {
        Self {
            bits_per_pixel,
            resolution,
            ptr,
        }
    }

    fn get() -> &'static Vram {
        VRAM.try_get().expect("VRAM not initialized")
    }
}

impl fmt::Display for Vram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}bpp Resolution: {}x{}",
            self.bits_per_pixel, self.resolution.x, self.resolution.y
        )
    }
}
