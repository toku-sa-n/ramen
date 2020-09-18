// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    device::pci::config::bar::Bar,
    mem::{
        allocator::{phys::FRAME_MANAGER, virt},
        paging::pml4::PML4,
    },
};
use core::ptr;
use x86_64::{
    structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, PhysFrame, Size4KiB},
    PhysAddr,
};
