// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use os_units::{Bytes, NumOfPages};
use x86_64::{
    instructions::{
        self, interrupts,
        port::{PortReadOnly, PortWriteOnly},
    },
    registers::model_specific::{Efer, EferFlags, LStar},
    structures::paging::Size4KiB,
    PhysAddr, VirtAddr,
};

use crate::mem::allocator;

pub fn init() {
    enable();
    register();
}

/// Safety: This function is unsafe because reading a value from I/O port may have side effects
/// which violate memory safety.
pub unsafe fn inb(port: u16) -> u8 {
    general_syscall(Syscalls::Inb, port.into(), 0)
        .try_into()
        .unwrap()
}

/// Safety: This function is unsafe because writing a value to I/O port may have side effects which
/// violate memory safety.
pub unsafe fn outb(port: u16, value: u8) {
    general_syscall(Syscalls::Outb, port.into(), value.into());
}

/// Safety: This function is unsafe because reading a value from I/O port may have side effects which violate memory safety.
pub unsafe fn inl(port: u16) -> u32 {
    general_syscall(Syscalls::Inl, port.into(), 0)
        .try_into()
        .unwrap()
}

/// Safety: This function is unsafe because writing a value via I/O port may have side effects
/// which violate memory safety.
pub unsafe fn outl(port: u16, value: u32) {
    general_syscall(Syscalls::Outl, port.into(), value.into());
}

pub fn halt() {
    // Safety: This operation is safe as it does not touch any unsafe things.
    unsafe { general_syscall(Syscalls::Halt, 0, 0) };
}

pub fn disable_interrupt() {
    // Safety: This operation is safe as it does not touch any unsafe things.
    unsafe { general_syscall(Syscalls::DisableInterrupt, 0, 0) };
}

pub fn enable_interrupt() {
    // Safety: This operation is safe as it does not touch any unsafe things.
    unsafe { general_syscall(Syscalls::EnableInterrupt, 0, 0) };
}

pub fn enable_interrupt_and_halt() {
    // Safety: This operation is safe as it does not touch any unsafe things.
    unsafe { general_syscall(Syscalls::EnableInterruptAndHalt, 0, 0) };
}

pub fn allocate_pages(pages: NumOfPages<Size4KiB>) -> VirtAddr {
    // Safety: This operation is safe as the arguments are propertly passed.
    VirtAddr::new(unsafe {
        general_syscall(
            Syscalls::AllocatePages,
            pages.as_usize().try_into().unwrap(),
            0,
        )
    })
}

pub fn deallocate_pages(virt: VirtAddr, pages: NumOfPages<Size4KiB>) {
    // Safety: This operation is safe as the all arguments are propertly passed.
    unsafe {
        general_syscall(
            Syscalls::DeallocatePages,
            virt.as_u64(),
            pages.as_usize().try_into().unwrap(),
        )
    };
}

pub fn map_pages(start: PhysAddr, bytes: Bytes) -> VirtAddr {
    // Safety: This operation is safe as the all arguments are propertly passed.
    VirtAddr::new(unsafe {
        general_syscall(
            Syscalls::MapPages,
            start.as_u64(),
            bytes.as_usize().try_into().unwrap(),
        )
    })
}

pub fn unmap_pages(start: VirtAddr, bytes: Bytes) {
    unsafe {
        general_syscall(
            Syscalls::UnmapPages,
            start.as_u64(),
            bytes.as_usize().try_into().unwrap(),
        );
    }
}

/// Safety: This function is unsafe if arguments are invalid.
unsafe fn general_syscall(ty: Syscalls, a1: u64, a2: u64) -> u64 {
    let ty = ty as u64;
    let r: u64;
    asm!("syscall",
        inout("rax") ty => r, inout("rdi") a1 => _, inout("rsi") a2 => _, out("rdx") _,
        out("rcx") _, out("r8") _, out("r9") _, out("r10") _, out("r11") _,);
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
/// RDI: 1st argument
/// RSI: 2nd argument
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
unsafe fn prepare_arguments() {
    let syscall_index: u64;
    let a1: u64;
    let a2: u64;

    asm!("", out("rax") syscall_index, out("rdi") a1, out("rsi") a2);
    asm!("", in("rax") select_proper_syscall(syscall_index, a1, a2))
}

/// Safety: This function is unsafe because invalid arguments may break memory safety.
unsafe fn select_proper_syscall(idx: u64, a1: u64, a2: u64) -> u64 {
    match FromPrimitive::from_u64(idx) {
        Some(s) => match s {
            Syscalls::Inb => sys_inb(a1.try_into().unwrap()).into(),
            Syscalls::Outb => sys_outb(a1.try_into().unwrap(), a2.try_into().unwrap()),
            Syscalls::Inl => sys_inl(a1.try_into().unwrap()).into(),
            Syscalls::Outl => sys_outl(a1.try_into().unwrap(), a2.try_into().unwrap()),
            Syscalls::Halt => sys_halt(),
            Syscalls::DisableInterrupt => sys_disable_interrupt(),
            Syscalls::EnableInterrupt => sys_enable_interrupt(),
            Syscalls::EnableInterruptAndHalt => sys_enable_interrupt_and_halt(),
            Syscalls::AllocatePages => {
                sys_allocate_pages(NumOfPages::new(a1.try_into().unwrap())).as_u64()
            }
            Syscalls::DeallocatePages => {
                sys_deallocate_pages(VirtAddr::new(a1), NumOfPages::new(a2.try_into().unwrap()))
            }
            Syscalls::MapPages => {
                sys_map_pages(PhysAddr::new(a1), Bytes::new(a2.try_into().unwrap())).as_u64()
            }
            Syscalls::UnmapPages => {
                sys_unmap_pages(VirtAddr::new(a1), Bytes::new(a2.try_into().unwrap()))
            }
        },
        None => panic!("Unsupported syscall index: {}", idx),
    }
}

/// Safety: This function is unsafe because reading from I/O port may have side effects which
/// violate memory safety.
unsafe fn sys_inb(port: u16) -> u8 {
    let mut p = PortReadOnly::new(port);
    p.read()
}

/// Safety: This function is unsafe because writing to I/O port may have side effects which violate
/// memory safety.
unsafe fn sys_outb(port: u16, v: u8) -> u64 {
    let mut p = PortWriteOnly::new(port);
    p.write(v);
    0
}

/// Safety: This function is unsafe because reading from I/O port may have side effects which
/// violate memory safety.
unsafe fn sys_inl(port: u16) -> u32 {
    let mut p = PortReadOnly::new(port);
    p.read()
}

/// Safety: This function is unsafe because writing to I/O port may have side effects which violate
/// memory safety.
unsafe fn sys_outl(port: u16, v: u32) -> u64 {
    let mut p = PortWriteOnly::new(port);
    p.write(v);
    0
}

fn sys_halt() -> u64 {
    instructions::hlt();
    0
}

fn sys_disable_interrupt() -> u64 {
    interrupts::disable();
    0
}

fn sys_enable_interrupt() -> u64 {
    interrupts::enable();
    0
}

fn sys_enable_interrupt_and_halt() -> u64 {
    interrupts::enable_and_hlt();
    0
}

fn sys_allocate_pages(num_of_pages: NumOfPages<Size4KiB>) -> VirtAddr {
    allocator::allocate_pages(num_of_pages)
}

fn sys_deallocate_pages(virt: VirtAddr, pages: NumOfPages<Size4KiB>) -> u64 {
    allocator::deallocate_pages(virt, pages);
    0
}

fn sys_map_pages(start: PhysAddr, bytes: Bytes) -> VirtAddr {
    crate::mem::map_pages(start, bytes)
}

fn sys_unmap_pages(start: VirtAddr, bytes: Bytes) -> u64 {
    crate::mem::unmap_pages(start, bytes);
    0
}

#[derive(FromPrimitive)]
enum Syscalls {
    Inb,
    Outb,
    Inl,
    Outl,
    Halt,
    DisableInterrupt,
    EnableInterrupt,
    EnableInterruptAndHalt,
    AllocatePages,
    DeallocatePages,
    MapPages,
    UnmapPages,
}
