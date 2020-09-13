// SPDX-License-Identifier: GPL-3.0-or-later

use {
    core::convert::TryFrom,
    os_units::{Bytes, Size},
    uefi::proto::console::gop,
    x86_64::PhysAddr,
};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Info {
    bpp: i32,
    screen_x: i32,
    screen_y: i32,
    ptr: PhysAddr,
}

impl Info {
    pub fn new_from_gop(gop: &mut gop::GraphicsOutput) -> Self {
        let (screen_x, screen_y) = gop.current_mode_info().resolution();

        Self {
            bpp: 32,
            screen_x: i32::try_from(screen_x)
                .expect("The width of screen resolution overflowed i32"),
            screen_y: i32::try_from(screen_y)
                .expect("The height of screen resolution overflowed i32"),
            ptr: PhysAddr::new(gop.frame_buffer().as_mut_ptr() as u64),
        }
    }

    #[must_use]
    pub fn bpp(&self) -> i32 {
        self.bpp
    }

    #[must_use]
    pub fn resolution(&self) -> (i32, i32) {
        (self.screen_x, self.screen_y)
    }

    #[must_use]
    pub fn phys_ptr(&self) -> PhysAddr {
        self.ptr
    }

    #[must_use]
    pub fn bytes(&self) -> Size<Bytes> {
        Size::new(
            usize::try_from(self.screen_x * self.screen_y * self.bpp / 8)
                .expect("The bytes of VRAM must not be negative"),
        )
    }
}
