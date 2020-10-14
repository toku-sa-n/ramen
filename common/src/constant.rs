// SPDX-License-Identifier: GPL-3.0-or-later

use {
    os_units::{Bytes, NumOfPages, Size},
    x86_64::{
        instructions::port::Port,
        structures::paging::{PageSize, Size4KiB},
        PhysAddr, VirtAddr,
    },
};

pub const LOCAL_APIC_ID_REGISTER_ADDR: PhysAddr = PhysAddr::new_truncate(0xfee0_0020);

pub const KERNEL_ADDR: VirtAddr = VirtAddr::new_truncate(0xffff_ffff_8000_0000);
pub const KERNEL_HEAP_ADDR: VirtAddr = VirtAddr::new_truncate(0xffff_ffff_9000_0000);
pub const FREE_PAGE_ADDR: VirtAddr = VirtAddr::new_truncate(0xffff_ffff_a000_0000); // Used by physical frame manager
pub const CHANGE_FREE_PAGE_ADDR: VirtAddr = {
    let offset = (FREE_PAGE_ADDR.as_u64() >> 12) & 0x1ff;
    let addr = 0xff80_0000_0000_0000
        | ((FREE_PAGE_ADDR.as_u64() >> 9) & 0xffff_ffff_ffff_f000)
        | (offset * 8);
    VirtAddr::new_truncate(addr)
};
pub const VRAM_ADDR: VirtAddr = VirtAddr::new_truncate(0xffff_ffff_a000_1000);
pub const STACK_BASE: VirtAddr = VirtAddr::new_truncate(0xffff_ffff_c000_0000);
pub const STACK_LOWER: VirtAddr =
    VirtAddr::new_truncate(STACK_BASE.as_u64() - NUM_OF_PAGES_STACK.as_bytes().as_usize() as u64);
pub const INIT_RSP: VirtAddr = VirtAddr::new_truncate(STACK_BASE.as_u64() - Size4KiB::SIZE);
pub const RECUR_PML4_ADDR: VirtAddr = VirtAddr::new_truncate(0xffff_ffff_ffff_f000);

pub const NUM_OF_PAGES_STACK: Size<NumOfPages<Size4KiB>> = Size::new(16);
pub const BYTES_KERNEL_HEAP: Size<Bytes> = Size::new(0x1000_0000);
pub const BYTES_AVAILABLE_RAM: Size<Bytes> = Size::new(0x1_0000_0000_0000);

pub const PORT_KEY_STATUS: Port<u8> = Port::new(0x0064);
pub const PORT_KEY_CMD: Port<u8> = Port::new(0x0064);
pub const PORT_KEY_DATA: Port<u8> = Port::new(0x0060);

pub const KEY_CMD_WRITE_MODE: u8 = 0x60;
pub const KEY_CMD_MODE: u8 = 0x47;
pub const KEY_STATUS_SEND_NOT_READY: u8 = 0x02;

pub const KERNEL_NAME: &str = "kernel.bin";
