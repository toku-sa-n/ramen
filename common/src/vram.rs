// SPDX-License-Identifier: GPL-3.0-or-later

use {
    core::convert::TryFrom, os_units::Bytes, uefi::proto::console::gop, vek::Vec2, x86_64::PhysAddr,
};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Info {
    bpp: i32,
    resolution: Vec2<i32>,
    ptr: PhysAddr,
}

impl Info {
    pub fn new_from_gop(gop: &mut gop::GraphicsOutput) -> Self {
        let resolution: Vec2<usize> = gop.current_mode_info().resolution().into();

        Self {
            bpp: 32,
            resolution: resolution.as_(),
            ptr: PhysAddr::new(gop.frame_buffer().as_mut_ptr() as u64),
        }
    }

    #[must_use]
    pub fn bpp(&self) -> i32 {
        self.bpp
    }

    #[must_use]
    pub fn resolution(&self) -> Vec2<i32> {
        self.resolution
    }

    #[must_use]
    pub fn phys_ptr(&self) -> PhysAddr {
        self.ptr
    }

    #[must_use]
    pub fn bytes(&self) -> Bytes {
        Bytes::new(
            usize::try_from(self.resolution.x * self.resolution.y * self.bpp / 8)
                .expect("The bytes of VRAM must not be negative"),
        )
    }
}
