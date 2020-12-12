// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;

pub fn syscall() {
    info!("This is `syscall` function.");
}

pub fn register() {
    let addr = syscall as u64;
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
