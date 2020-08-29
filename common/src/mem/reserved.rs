// SPDX-License-Identifier: GPL-3.0-or-later

use crate::constant::{KERNEL_ADDR, NUM_OF_PAGES_STACK, STACK_LOWER, VRAM_ADDR};
use crate::size::{Byte, Size};
use crate::vram;
use x86_64::{PhysAddr, VirtAddr};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Map([Range; 3]);
impl Map {
    pub fn new(
        phys_addr_kernel: PhysAddr,
        bytes_kernel: Size<Byte>,
        phys_addr_stack: PhysAddr,
        vram: &vram::Info,
    ) -> Self {
        Self {
            0: [
                Range::kernel(phys_addr_kernel, bytes_kernel),
                Range::stack(phys_addr_stack),
                Range::vram(vram),
            ],
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Range> {
        self.0.iter()
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Range {
    virt: VirtAddr,
    phys: PhysAddr,
    bytes: Size<Byte>,
}

impl Range {
    pub fn kernel(phys: PhysAddr, bytes: Size<Byte>) -> Self {
        Self {
            virt: KERNEL_ADDR,
            phys,
            bytes,
        }
    }

    pub fn vram(vram: &vram::Info) -> Self {
        Self {
            virt: VRAM_ADDR,
            phys: vram.phys_ptr(),
            bytes: vram.bytes(),
        }
    }

    pub fn stack(phys: PhysAddr) -> Self {
        Self {
            virt: STACK_LOWER,
            phys,
            bytes: NUM_OF_PAGES_STACK.as_bytes(),
        }
    }

    pub fn virt(&self) -> VirtAddr {
        self.virt
    }

    pub fn phys(&self) -> PhysAddr {
        self.phys
    }

    pub fn bytes(&self) -> Size<Byte> {
        self.bytes
    }
}
