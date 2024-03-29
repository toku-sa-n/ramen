#![no_std]

use {
    conquer_once::spin::Lazy,
    os_units::{Bytes, NumOfPages},
    x86_64::{structures::paging::Size4KiB, VirtAddr},
};

pub const KERNEL_ADDR: VirtAddr = VirtAddr::new_truncate(0xffff_ffff_8000_0000);
pub const INITRD_ADDR: VirtAddr = VirtAddr::new_truncate(0xffff_ffff_8800_0000);
pub const VRAM_ADDR: VirtAddr = VirtAddr::new_truncate(0xffff_ffff_a000_1000);
pub const STACK_BASE: VirtAddr = VirtAddr::new_truncate(0xffff_ffff_c000_0000);
pub static STACK_LOWER: Lazy<VirtAddr> = Lazy::new(|| {
    VirtAddr::new_truncate(STACK_BASE.as_u64() - NUM_OF_PAGES_STACK.as_bytes().as_usize() as u64)
});
pub static INTERRUPT_STACK: Lazy<VirtAddr> =
    Lazy::new(|| STACK_BASE - NUM_OF_PAGES_STACK.as_bytes().as_usize() / 2);
pub const RECUR_PML4_ADDR: VirtAddr = VirtAddr::new_truncate(0xff7f_bfdf_e000);

pub static NUM_OF_PAGES_STACK: Lazy<NumOfPages<Size4KiB>> = Lazy::new(|| NumOfPages::new(16));
pub const BYTES_AVAILABLE_RAM: Bytes = Bytes::new(0x1_0000_0000_0000);
