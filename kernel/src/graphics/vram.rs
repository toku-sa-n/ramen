// SPDX-License-Identifier: GPL-3.0-or-later

use common::{constant::VRAM_ADDR, kernelboot};
use conquer_once::spin::{Lazy, OnceCell};
use screen_layer::Vec2;
use x86_64::VirtAddr;

static VRAM: Lazy<OnceCell<Vram>> = Lazy::new(OnceCell::uninit);

pub fn init(boot_info: &kernelboot::Info) {
    VRAM.try_init_once(|| Vram::new_from_boot_info(boot_info))
        .expect("`VRAM` is initialized more than once.");
}

pub fn resolution() -> Vec2<u32> {
    vram().resolution()
}

pub fn bpp() -> u32 {
    vram().bpp()
}

pub fn ptr() -> VirtAddr {
    vram().ptr()
}

pub fn print_info() {
    let r = resolution();
    info!("{}bpp Resolution: {}x{}", bpp(), r.y, r.y)
}

fn vram() -> &'static Vram {
    VRAM.try_get().expect("`VRAM` is not initialized.")
}

struct Vram {
    bits_per_pixel: u32,
    resolution: Vec2<u32>,
    ptr: VirtAddr,
}
impl Vram {
    fn resolution(&self) -> Vec2<u32> {
        self.resolution
    }

    fn bpp(&self) -> u32 {
        self.bits_per_pixel
    }

    fn ptr(&self) -> VirtAddr {
        self.ptr
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
}
