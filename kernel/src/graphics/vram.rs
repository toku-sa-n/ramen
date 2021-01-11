// SPDX-License-Identifier: GPL-3.0-or-later

use super::font::FONT_HEIGHT;
use common::{constant::VRAM_ADDR, kernelboot};
use conquer_once::spin::OnceCell;
use core::{
    convert::{TryFrom, TryInto},
    ops::{Index, IndexMut},
    slice,
};
use rgb::RGB8;
use spinning_top::{Spinlock, SpinlockGuard};
use vek::Vec2;

static VRAM: Spinlock<Vram> = Spinlock::new(Vram);
static INFO: OnceCell<Info> = OnceCell::uninit();

pub fn init(boot_info: &kernelboot::Info) {
    init_info(boot_info);
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

pub(super) fn scroll_up() {
    lock().scroll_up();
}

pub(super) fn set_color(coord: Vec2<usize>, color: RGB8) {
    let (x, y) = coord.into_tuple();

    lock()[y][x] = color.into();
}

fn lock() -> SpinlockGuard<'static, Vram> {
    VRAM.try_lock()
        .expect("Failed to acquire the lock of `VRAM`")
}

fn init_info(boot_info: &kernelboot::Info) {
    INFO.try_init_once(|| Info::new_from_boot_info(boot_info))
        .expect("`INFO` is initialized more than once.");
}

fn clear_screen() {
    lock().clear();
}

fn info() -> &'static Info {
    INFO.try_get().expect("`INFO` is not initialized.")
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

struct Vram;
impl Vram {
    fn clear(&mut self) {
        let (x, y): (usize, usize) = resolution().as_().into_tuple();
        for y in 0..y {
            for x in 0..x {
                self[y][x] = Bgr::default();
            }
        }
    }

    fn scroll_up(&mut self) {
        let fh: usize = FONT_HEIGHT.try_into().unwrap();
        let (w, h): (usize, usize) = resolution().as_().into_tuple();
        let lc = h / fh;
        let log_bottom = fh * (lc - 1);

        for x in 0..w {
            for y in 0..log_bottom {
                self[y][x] = self[y + fh][x];
            }

            for y in log_bottom..h {
                self[y][x] = Bgr::default();
            }
        }
    }
}
impl Index<usize> for Vram {
    type Output = [Bgr];

    fn index(&self, index: usize) -> &Self::Output {
        let p = VRAM_ADDR + index * usize::try_from(resolution().x * bpp() / 8).unwrap();

        unsafe { slice::from_raw_parts(p.as_ptr(), resolution().x.try_into().unwrap()) }
    }
}
impl IndexMut<usize> for Vram {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let p = VRAM_ADDR + index * usize::try_from(resolution().x * bpp() / 8).unwrap();

        unsafe { slice::from_raw_parts_mut(p.as_mut_ptr(), resolution().x.try_into().unwrap()) }
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Default)]
struct Bgr {
    b: u8,
    g: u8,
    r: u8,
    _alpha: u8,
}
impl From<RGB8> for Bgr {
    fn from(rgb: RGB8) -> Self {
        Self {
            b: rgb.b,
            g: rgb.g,
            r: rgb.r,
            _alpha: 0,
        }
    }
}
