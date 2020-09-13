// SPDX-License-Identifier: GPL-3.0-or-later

use crate::constant::{
    BYTES_KERNEL_HEAP, FREE_PAGE_ADDR, KERNEL_ADDR, KERNEL_HEAP_ADDR, NUM_OF_PAGES_STACK,
    STACK_LOWER, VRAM_ADDR,
};
use crate::vram;
use os_units::{Bytes, Size};
use x86_64::{PhysAddr, VirtAddr};

pub struct KernelPhysRange {
    start: PhysAddr,
    bytes: Size<Bytes>,
}

impl KernelPhysRange {
    #[must_use]
    pub fn new(start: PhysAddr, bytes: Size<Bytes>) -> Self {
        Self { start, bytes }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Map([Range; 5]);
impl Map {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        kernel: &KernelPhysRange,
        phys_addr_stack: PhysAddr,
        vram: &vram::Info,
        heap: PhysAddr,
        free_page: PhysAddr,
    ) -> Self {
        Self {
            0: [
                Range::kernel(&kernel),
                Range::stack(phys_addr_stack),
                Range::vram(vram),
                Range::heap(heap),
                Range::free_page(free_page),
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
    bytes: Size<Bytes>,
}

impl Range {
    #[must_use]
    fn kernel(kernel: &KernelPhysRange) -> Self {
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
    fn heap(phys: PhysAddr) -> Self {
        Self {
            virt: KERNEL_HEAP_ADDR,
            phys,
            bytes: BYTES_KERNEL_HEAP,
        }
    }

    #[must_use]
    fn free_page(phys: PhysAddr) -> Self {
        Self {
            virt: FREE_PAGE_ADDR,
            phys,
            bytes: Size::new(0x1000),
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
    pub fn bytes(&self) -> Size<Bytes> {
        self.bytes
    }
}
