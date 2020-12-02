// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    constant::{INITRD_ADDR, KERNEL_ADDR, NUM_OF_PAGES_STACK, STACK_LOWER, VRAM_ADDR},
    vram,
};
use os_units::Bytes;
use x86_64::{PhysAddr, VirtAddr};

pub struct PhysRange {
    start: PhysAddr,
    bytes: Bytes,
}

impl PhysRange {
    #[must_use]
    pub fn new(start: PhysAddr, bytes: Bytes) -> Self {
        Self { start, bytes }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Map([Range; 4]);
impl Map {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        kernel: &PhysRange,
        phys_addr_stack: PhysAddr,
        vram: &vram::Info,
        initrd: &PhysRange,
    ) -> Self {
        Self {
            0: [
                Range::kernel(&kernel),
                Range::stack(phys_addr_stack),
                Range::vram(vram),
                Range::initrd(initrd),
            ],
        }
    }

    #[must_use]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &Range> {
        self.0.iter()
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Range {
    virt: VirtAddr,
    phys: PhysAddr,
    bytes: Bytes,
}

impl Range {
    #[must_use]
    fn kernel(kernel: &PhysRange) -> Self {
        Self {
            virt: KERNEL_ADDR,
            phys: kernel.start,
            bytes: kernel.bytes,
        }
    }

    #[must_use]
    fn vram(vram: &vram::Info) -> Self {
        Self {
            virt: VRAM_ADDR,
            phys: vram.phys_ptr(),
            bytes: vram.bytes(),
        }
    }

    #[must_use]
    fn stack(phys: PhysAddr) -> Self {
        Self {
            virt: STACK_LOWER,
            phys,
            bytes: NUM_OF_PAGES_STACK.as_bytes(),
        }
    }

    #[must_use]
    fn initrd(initrd: &PhysRange) -> Self {
        Self {
            virt: INITRD_ADDR,
            phys: initrd.start,
            bytes: initrd.bytes,
        }
    }

    #[must_use]
    pub fn virt(&self) -> VirtAddr {
        self.virt
    }

    #[must_use]
    pub fn phys(&self) -> PhysAddr {
        self.phys
    }

    #[must_use]
    pub fn bytes(&self) -> Bytes {
        self.bytes
    }
}
