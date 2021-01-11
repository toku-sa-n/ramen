// SPDX-License-Identifier: GPL-3.0-or-later

use common::{constant::VRAM_ADDR, kernelboot};
use conquer_once::spin::{Lazy, OnceCell};
use core::ptr;
use rgb::RGB8;
use vek::Vec2;
use x86_64::VirtAddr;

static INFO: Lazy<OnceCell<Info>> = Lazy::new(OnceCell::uninit);

pub fn init(boot_info: &kernelboot::Info) {
    VRAM.try_init_once(|| Info::new_from_boot_info(boot_info))
        .expect("`VRAM` is initialized more than once.");
}

pub fn resolution() -> Vec2<u32> {
    info().resolution()
}

pub fn bpp() -> u32 {
    info().bpp()
}

pub fn ptr() -> VirtAddr {
    info().ptr()
}

pub fn print_info() {
    let r = resolution();
    info!("{}bpp Resolution: {}x{}", bpp(), r.y, r.y)
}

pub(super) fn set_pixel(coord: Vec2<u32>, color: RGB8) {
    assert!(
        coord.partial_cmplt(&resolution()).reduce_and(),
        "`coord` is outsid the screen."
    );

    let r = resolution();
    let offset = (coord.y * r.x + coord.x) * bpp() / 8;
    let p = ptr().as_u64() + u64::from(offset);

    unsafe {
        ptr::write_volatile(p as *mut u8, color.b);
        ptr::write_volatile((p + 1) as *mut u8, color.g);
        ptr::write_volatile((p + 2) as *mut u8, color.r);
    }
}

fn info() -> &'static Info {
    VRAM.try_get().expect("`VRAM` is not initialized.")
}

struct Info {
    bits_per_pixel: u32,
    resolution: Vec2<u32>,
    ptr: VirtAddr,
}
impl Info {
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
