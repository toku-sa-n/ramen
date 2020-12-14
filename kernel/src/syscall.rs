// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use x86_64::{
    instructions::port::PortReadOnly,
    registers::model_specific::{Efer, EferFlags, LStar},
    VirtAddr,
};

pub fn init() {
    enable();
    register();
}

pub fn read_from_port(port: u16) -> u32 {
    let r: u32;
    const R: u64 = Syscalls::ReadFromPort as u64;
    unsafe {
        asm!("
            mov rax, {}
            mov ebx, {:e}
            syscall
            mov {:e}, eax
            ", const R, in(reg) u32::from(port), out(reg) r);
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

        call prepare_arguments

        pop r11     # Restore rflags
        pop rcx     # Restore rip
        sti
        sysretq
        ",
            options(noreturn)
        );
    }
}

/// Safety: This function is unsafe because invalid values in registers may break memory safety.
#[no_mangle]
unsafe fn prepare_arguments() -> u64 {
    let syscall_index: u64;
    let a1: u64;
    let a2: u64;

    asm!("
        mov {}, rax
        mov {}, rbx
        mov {}, rdx
        ", out(reg) syscall_index, out(reg) a1, out(reg) a2);

    select_proper_syscall(syscall_index, a1, a2)
}

/// Safety: This function is unsafe because invalid arguments may break memory safety.
unsafe fn select_proper_syscall(idx: u64, a1: u64, a2: u64) -> u64 {
    match FromPrimitive::from_u64(idx) {
        Some(s) => match s {
            Syscalls::ReadFromPort => sys_read_from_port(a1.try_into().unwrap()).into(),
        },
        None => panic!("Unsupported syscall index: {}", idx),
    }
}

/// Safety: This function is unsafe because reading from I/O port may have side effects which
/// violate memory safety.
unsafe fn sys_read_from_port(port: u16) -> u32 {
    let mut p = PortReadOnly::new(port);
    p.read()
}

#[derive(FromPrimitive)]
enum Syscalls {
    ReadFromPort = 1,
}
