// SPDX-License-Identifier: GPL-3.0-or-later

use common::{constant::VRAM_ADDR, kernelboot};
use conquer_once::spin::OnceCell;
use core::{
    convert::{TryFrom, TryInto},
    slice,
};
use rgb::RGB8;
use spinning_top::{Spinlock, SpinlockGuard};
use vek::Vec2;

use super::font::FONT_HEIGHT;

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

pub(super) fn scroll_up() {
    vram().scroll_up();
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

struct Vram(&'static mut [Bgr]);
impl Vram {
    fn new() -> Self {
        let len = resolution().product();
        let buf =
            unsafe { slice::from_raw_parts_mut(VRAM_ADDR.as_mut_ptr(), len.try_into().unwrap()) };

        Self(buf)
    }

    fn clear(&mut self) {
        for c in self.0.iter_mut() {
            *c = Bgr::default();
        }
    }

    fn set_pixel(&mut self, coord: Vec2<u32>, color: RGB8) {
        assert!(
            coord.partial_cmplt(&resolution()).reduce_and(),
            "`coord` is outsid the screen."
        );

        let r = resolution();
        let index: usize = (coord.y * r.x + coord.x).try_into().unwrap();
        self.0[index] = color.into();
    }

    fn scroll_up(&mut self) {
        let fh: usize = FONT_HEIGHT.try_into().unwrap();
        let (w, h): (usize, usize) = resolution().as_().into_tuple();
        let lc = h / fh;
        let log_bottom = fh * (lc - 1) * w;
        let offset = fh * w;

        for i in 0..log_bottom {
            self.0[i] = self.0[i + offset];
        }

        for i in log_bottom..self.0.len() {
            self.0[i] = Bgr::default();
        }
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
