use {core::convert::TryFrom, os_units::Bytes, vek::Vec2, x86_64::PhysAddr};

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Info {
    bpp: u32,
    resolution: Vec2<u32>,
    ptr: PhysAddr,
}

impl Info {
    #[must_use]
    pub fn new(bpp: u32, resolution: Vec2<u32>, ptr: PhysAddr) -> Self {
        Self {
            bpp,
            resolution,
            ptr,
        }
    }

    #[must_use]
    pub fn bpp(&self) -> u32 {
        self.bpp
    }

    #[must_use]
    pub fn resolution(&self) -> Vec2<u32> {
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
