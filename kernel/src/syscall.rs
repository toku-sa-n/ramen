// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;
use x86_64::instructions::interrupts;

pub fn init() {
    enable();
    register();
}

fn enable() {
    unsafe {
        asm!(
            "
        mov ecx, 0xc0000080
        rdmsr
        or eax, 1
        wrmsr
        "
        );
    }
}

fn register() {
    let addr = wrapper as usize;
    let l: u32 = (addr & 0xffff_ffff).try_into().unwrap();
    let u: u32 = (addr >> 32).try_into().unwrap();

    unsafe {
        asm!("
        mov edx, {:e}
        mov eax, {:e}
        mov ecx, 0xc0000082
        wrmsr
        ",in(reg) u,in(reg) l);
    }
}

fn wrapper() {
    interrupts::disable();
    unsafe {
        asm!(
            "
        push rcx    # Save rip
        push r11    # Save rflags
        "
        );
    }

    syscall();

    unsafe {
        asm!(
            "
        pop r11     # Restore rflags
        pop rcx     # Restore rip
        sti
        sysret
        "
        );
    }
}

fn syscall() {
    info!("This is `syscall` function.");
}
