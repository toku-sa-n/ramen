// SPDX-License-Identifier: GPL-3.0-or-later

use crate::size::{Byte, Size};
use uefi::proto::console::gop;
use x86_64::PhysAddr;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Info {
    bpp: usize,
    screen_x: usize,
    screen_y: usize,
    ptr: PhysAddr,
}

impl Info {
    pub fn new_from_gop(gop: &mut gop::GraphicsOutput) -> Self {
        let (screen_x, screen_y) = gop.current_mode_info().resolution();

        Self {
            bpp: 32,
            screen_x: screen_x,
            screen_y: screen_y,
            ptr: PhysAddr::new(gop.frame_buffer().as_mut_ptr() as u64),
        }
    }

    pub fn bpp(&self) -> usize {
        self.bpp
    }

    pub fn resolution(&self) -> (usize, usize) {
        (self.screen_x, self.screen_y)
    }

    pub fn phys_ptr(&self) -> PhysAddr {
        self.ptr
    }

    pub fn bytes(&self) -> Size<Byte> {
        Size::new(self.screen_x as usize * self.screen_y as usize * self.bpp as usize / 8)
    }
}
