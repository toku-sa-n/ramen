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

    pub fn display() -> impl core::fmt::Display {
        Self::get()
    }

    pub unsafe fn set_color(coord: &Vec2<isize>, rgb: RGB8) {
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

impl fmt::Display for Vram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}bpp Resolution: {}x{}",
            self.bits_per_pixel, self.resolution.x, self.resolution.y
        )
    }
}
