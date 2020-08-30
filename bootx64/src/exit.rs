// SPDX-License-Identifier: GPL-3.0-or-later

use common::boot as kernelboot;
use common::constant::INIT_RSP;
use x86_64::VirtAddr;

pub fn bootx64<'a>(entry_addr: VirtAddr, boot_info: kernelboot::Info) -> ! {
    disable_interruption();

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

fn jump_to_kernel(boot_info: kernelboot::Info, entry_addr: VirtAddr) -> ! {
    boot_info.set();

    unsafe {
        asm!("mov rsp, rax
        jmp rdi",in("rax") INIT_RSP.as_u64(),in("rdi") entry_addr.as_u64(),options(nomem, preserves_flags, nostack,noreturn));
    }
}
