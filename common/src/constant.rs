// SPDX-License-Identifier: GPL-3.0-or-later

use crate::size::{NumOfPages, Size};
use crate::x86_64::VirtAddr;
use x86_64::structures::paging::{PageSize, Size4KiB};

pub const KERNEL_ADDR: VirtAddr = VirtAddr::new_truncate(0xffff_ffff_8000_0000);
pub const VRAM_ADDR: VirtAddr = VirtAddr::new_truncate(0xffff_ffff_a000_0000);
pub const STACK_BASE: VirtAddr = VirtAddr::new_truncate(0xffff_ffff_c000_0000);
pub const STACK_LOWER: VirtAddr =
    VirtAddr::new_truncate(STACK_BASE.as_u64() - NUM_OF_PAGES_STACK.as_bytes().as_usize() as u64);
pub const INIT_RSP: VirtAddr = VirtAddr::new_truncate(STACK_BASE.as_u64() - Size4KiB::SIZE);
pub const RECUR_PML4_ADDR: VirtAddr = VirtAddr::new_truncate(0xffff_ffff_ffff_f000);

pub const NUM_OF_PAGES_STACK: Size<NumOfPages> = Size::new(16);

pub const KERNEL_NAME: &'static str = "kernel.bin";
