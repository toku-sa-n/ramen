// SPDX-License-Identifier: GPL-3.0-or-later

use {
    boot_info::vram,
    os_units::Bytes,
    predefined_mmap::{KERNEL_ADDR, NUM_OF_PAGES_STACK, STACK_LOWER, VRAM_ADDR},
    x86_64::{PhysAddr, VirtAddr},
};

#[derive(Copy, Clone, Debug)]
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
#[derive(Copy, Clone, Debug)]
pub struct Map([Range; 3]);
impl Map {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(kernel: &PhysRange, phys_addr_stack: PhysAddr, vram: &vram::Info) -> Self {
        Self {
            0: [
                Range::kernel(kernel),
                Range::stack(phys_addr_stack),
                Range::vram(vram),
            ],
        }
    }

    #[must_use]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &Range> {
        self.0.iter()
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
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
            virt: *STACK_LOWER,
            phys,
            bytes: NUM_OF_PAGES_STACK.as_bytes(),
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
