// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use x86_64::{
    instructions::port::{PortReadOnly, PortWriteOnly},
    registers::model_specific::{Efer, EferFlags, LStar},
    VirtAddr,
};

pub fn init() {
    enable();
    register();
}

/// Safety: This function is unsafe because reading a value from I/O port may have side effects
/// which violate memory safety.
pub unsafe fn read_from_port(port: u16) -> u32 {
    let r: u32;
    const R: u64 = Syscalls::ReadFromPort as u64;
    asm!("
            mov rax, {}
            mov ebx, {:e}
            syscall
            mov {:e}, eax
            ", const R, in(reg) u32::from(port), out(reg) r);
    r
}

/// Safety: This function is unsafe because writing a value via I/O port may have side effects
/// which violate memory safety.
pub unsafe fn write_to_port(port: u16, value: u32) {
    const R: u64 = Syscalls::WriteToPort as u64;
    asm!("
        mov rax, {}
        mov ebx, {:e}
        mov edx, {:e}
        syscall
        ", const R, in(reg) u32::from(port), in(reg) value);
}

pub fn disable_interrupt() {
    const R: u64 = Syscalls::DisableInterrupt as u64;

    // Safety: This operation is safe as it does not touch any unsafe things.
    unsafe { asm!("syscall", in("rax") R) }
}

pub fn enable_interrupt() {
    const R: u64 = Syscalls::EnableInterrupt as u64;

    // Safety: This operation is safe as it does not touch any unsafe things.
    unsafe {
        asm!("
        mov rax, {}
        syscall
        ", const R);
    }
}

pub fn enable_interrupt_and_halt() {
    const R: u64 = Syscalls::EnableInterruptAndHalt as u64;

    // Safety: This operation is safe as it does not touch any unsafe things.
    unsafe {
        asm!("
        mov rax, {}
        syscall
        ", const R);
    }
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
            Syscalls::WriteToPort => {
                sys_write_to_port(a1.try_into().unwrap(), a2.try_into().unwrap())
            }
            Syscalls::DisableInterrupt => sys_disable_interrupt(),
            Syscalls::EnableInterrupt => sys_enable_interrupt(),
            Syscalls::EnableInterruptAndHalt => sys_enable_interrupt_and_halt(),
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

/// Safety: This function is unsafe because writing to I/O port may have side effects which violate
/// memory safety.
unsafe fn sys_write_to_port(port: u16, v: u32) -> u64 {
    let mut p = PortWriteOnly::new(port);
    p.write(v);
    0
}

fn sys_disable_interrupt() -> u64 {
    x86_64::instructions::interrupts::disable();
    0
}

fn sys_enable_interrupt() -> u64 {
    x86_64::instructions::interrupts::enable();
    0
}

fn sys_enable_interrupt_and_halt() -> u64 {
    x86_64::instructions::interrupts::enable_and_hlt();
    0
}

#[derive(FromPrimitive)]
enum Syscalls {
    ReadFromPort,
    WriteToPort,
    DisableInterrupt,
    EnableInterrupt,
    EnableInterruptAndHalt,
}
