use crate::mem::paging;
use common_items::constant::{INIT_RSP, KERNEL_ADDR};
use common_items::size::{Byte, Size};
use core::ptr;
use uefi::table::boot;
use x86_64::{PhysAddr, VirtAddr};

pub fn bootx64<'a>(
    mem_map: &'a mut [boot::MemoryDescriptor],
    boot_info: common_items::BootInfo,
    kernel_addr: PhysAddr,
    bytes_kernel: Size<Byte>,
    stack_addr: PhysAddr,
) -> ! {
    disable_interruption();

    paging::init(
        mem_map,
        &boot_info.vram(),
        kernel_addr,
        bytes_kernel,
        stack_addr,
    );
    jump_to_kernel(boot_info);
}

fn disable_interruption() -> () {
    // Use `nop` because some machines go wrong when continuously doing `out`.
    unsafe {
        asm!(
            "mov al,0xff
            out 0x21,al
            nop
            out 0xa1,al
            cli"
        );
    }
}

fn jump_to_kernel(boot_info: common_items::BootInfo) -> ! {
    boot_info.set();

    unsafe {
        asm!("mov rsp, rax
        jmp rdi",in("rax") INIT_RSP.as_u64(),in("rdi") fetch_entry_address().as_u64(),options(nomem, preserves_flags, nostack,noreturn));
    }
}

fn fetch_entry_address() -> VirtAddr {
    VirtAddr::new(unsafe { ptr::read(KERNEL_ADDR.as_ptr::<u64>()) })
}
