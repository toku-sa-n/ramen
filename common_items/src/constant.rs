use crate::size::{NumOfPages, Size};
use crate::x86_64::VirtAddr;
use crate::BootInfo;
use core::mem::size_of;

pub const KERNEL_ADDR: VirtAddr = VirtAddr::new_truncate(0xffff_ffff_8000_0000);
pub const PML4_ADDR: VirtAddr = VirtAddr::new_truncate(0xffff_ffff_a000_0000);
pub const VRAM_ADDR: VirtAddr = VirtAddr::new_truncate(0xffff_ffff_a000_1000);
pub const STACK_BASE: VirtAddr = VirtAddr::new_truncate(0xffff_ffff_c000_0000);
pub const INIT_RSP: VirtAddr =
    VirtAddr::new_truncate(STACK_BASE.as_u64() - size_of::<BootInfo>() as u64);
pub const RECUR_PML4_ADDR: VirtAddr = VirtAddr::new_truncate(0xffff_ff80_0000_0000);

pub const NUM_OF_PAGES_STACK: Size<NumOfPages> = Size::new(16);

pub const KERNEL_NAME: &'static str = "kernel.bin";
