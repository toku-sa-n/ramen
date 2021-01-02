// SPDX-License-Identifier: GPL-3.0-or-later

pub mod font;

#[macro_use]
pub mod screen;

use common::{constant::VRAM_ADDR, kernelboot};
use conquer_once::spin::{Lazy, OnceCell};
use core::fmt;
use vek::Vec2;
use x86_64::VirtAddr;

static VRAM: Lazy<OnceCell<Vram>> = Lazy::new(OnceCell::uninit);

#[derive(Clone)]
pub struct Vram {
    bits_per_pixel: u32,
    resolution: Vec2<u32>,
    ptr: VirtAddr,
}

impl Vram {
    pub fn init(boot_info: &kernelboot::Info) {
        VRAM.try_init_once(|| Self::new_from_boot_info(boot_info))
            .unwrap();
    }

    pub fn resolution() -> &'static Vec2<u32> {
        &Vram::get().resolution
    }

    pub fn display() -> impl core::fmt::Display {
        Self::get()
    }

    pub fn bpp() -> u32 {
        Vram::get().bits_per_pixel
    }

    pub fn ptr() -> VirtAddr {
        Vram::get().ptr
    }

    fn new_from_boot_info(boot_info: &kernelboot::Info) -> Self {
        let vram = boot_info.vram();

        Self::new(vram.bpp(), vram.resolution(), VRAM_ADDR)
    }

    fn new(bits_per_pixel: u32, resolution: Vec2<u32>, ptr: VirtAddr) -> Self {
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
