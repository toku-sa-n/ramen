use crate::mem::paging;
use common_items::constant::INIT_RSP;
use common_items::size::{Byte, Size};
use uefi::table::boot;
use x86_64::{PhysAddr, VirtAddr};

pub fn bootx64<'a>(
    mem_map: &'a mut [boot::MemoryDescriptor],
    boot_info: common_items::BootInfo,
    entry_addr: VirtAddr,
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
    jump_to_kernel(boot_info, entry_addr);
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

fn jump_to_kernel(boot_info: common_items::BootInfo, entry_addr: VirtAddr) -> ! {
    boot_info.set();

    unsafe {
        asm!("mov rsp, rax
        jmp rdi",in("rax") INIT_RSP.as_u64(),in("rdi") entry_addr.as_u64(),options(nomem, preserves_flags, nostack,noreturn));
    }
}
