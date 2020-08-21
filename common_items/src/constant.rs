use crate::x86_64::VirtAddr;

pub const KERNEL_ADDR: VirtAddr = VirtAddr::new_truncate(0xffff_ffff_8000_0000);
