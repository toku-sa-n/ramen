// SPDX-License-Identifier: GPL-3.0-or-later

use common::{constant::VRAM_ADDR, kernelboot};
use conquer_once::spin::OnceCell;
use core::{convert::TryInto, slice};
use rgb::RGB8;
use spinning_top::{Spinlock, SpinlockGuard};
use vek::Vec2;

static INFO: OnceCell<Info> = OnceCell::uninit();
static VRAM: OnceCell<Spinlock<Vram>> = OnceCell::uninit();

pub fn init(boot_info: &kernelboot::Info) {
    init_info(boot_info);
    init_vram();
    clear_screen();
}

pub fn resolution() -> Vec2<u32> {
    info().resolution()
}

pub fn bpp() -> u32 {
    info().bpp()
}

pub fn print_info() {
    let r = resolution();
    info!("{}bpp Resolution: {}x{}", bpp(), r.y, r.y)
}

pub(super) fn set_pixel(coord: Vec2<u32>, color: RGB8) {
    vram().set_pixel(coord, color);
}

fn init_info(boot_info: &kernelboot::Info) {
    INFO.try_init_once(|| Info::new_from_boot_info(boot_info))
        .expect("`INFO` is initialized more than once.");
}

fn init_vram() {
    VRAM.try_init_once(|| Spinlock::new(Vram::new()))
        .expect("`VRAM` is initialized more than once.");
}

fn clear_screen() {
    vram().clear();
}

fn info() -> &'static Info {
    INFO.try_get().expect("`INFO` is not initialized.")
}

fn vram() -> SpinlockGuard<'static, Vram> {
    let v = VRAM.try_get().expect("`VRAM` is not initialized.");
    v.try_lock().expect("Failed to acquire the lock of `VRAM`.")
}

struct Info {
    bits_per_pixel: u32,
    resolution: Vec2<u32>,
}
impl Info {
    fn resolution(&self) -> Vec2<u32> {
        self.resolution
    }

    fn bpp(&self) -> u32 {
        self.bits_per_pixel
    }

    fn new_from_boot_info(boot_info: &kernelboot::Info) -> Self {
        let vram = boot_info.vram();

        Self::new(vram.bpp(), vram.resolution())
    }

    fn new(bits_per_pixel: u32, resolution: Vec2<u32>) -> Self {
        Self {
            bits_per_pixel,
            resolution,
        }
    }
}

struct Vram(&'static mut [u8]);
impl Vram {
    fn new() -> Self {
        let len = resolution().product() + bpp() / 8;
        let buf =
            unsafe { slice::from_raw_parts_mut(VRAM_ADDR.as_mut_ptr(), len.try_into().unwrap()) };

        Self(buf)
    }

    fn clear(&mut self) {
        for c in self.0.iter_mut() {
            *c = 0;
        }
    }

    fn set_pixel(&mut self, coord: Vec2<u32>, color: RGB8) {
        assert!(
            coord.partial_cmplt(&resolution()).reduce_and(),
            "`coord` is outsid the screen."
        );

        let r = resolution();
        let index: usize = ((coord.y * r.x + coord.x) * bpp() / 8).try_into().unwrap();
        self.0[index] = color.b;
        self.0[index + 1] = color.g;
        self.0[index + 2] = color.r;
    }
}
