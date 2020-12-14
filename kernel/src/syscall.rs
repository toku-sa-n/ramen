// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;
use x86_64::{
    registers::model_specific::{Efer, EferFlags, LStar},
    VirtAddr,
};

pub fn init() {
    enable();
    register();
}

pub fn read_from_port(port: u16) -> u32 {
    let r: u32;
    unsafe {
        asm!("
            mov eax, 0
            mov ebx, {:e}
            syscall
            mov {:e}, eax
            ", in(reg) u32::from(port), out(reg) r);
    }
    r
}

fn enable() {
    // Safety: This operation is safe as this does not touch any unsafe things.
    unsafe { Efer::update(|e| *e |= EferFlags::SYSTEM_CALL_EXTENSIONS) }
}

fn register() {
    let addr = save_rip_and_rflags as usize;

    LStar::write(VirtAddr::new(addr.try_into().unwrap()));
}

/// `syscall` instruction calls this function.
///
/// RAX: system call index
/// RBX: 1st argument
/// RDX: 2nd argument
#[naked]
extern "C" fn save_rip_and_rflags() -> u64 {
    unsafe {
        asm!(
            "
        cli
        push rcx    # Save rip
        push r11    # Save rflags

        call syscall

        pop r11     # Restore rflags
        pop rcx     # Restore rip
        sti
        sysretq
        ",
            options(noreturn)
        );
    }
}

#[no_mangle]
fn syscall() -> u64 {
    let syscall_index: u64;
    let a1: u64;
    let a2: u64;

    unsafe {
        asm!("
        mov {}, rax
        mov {}, rbx
        mov {}, rdx
        ", out(reg) syscall_index, out(reg) a1, out(reg) a2);
    }

    334
}
