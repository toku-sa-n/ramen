// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![feature(asm)]

use core::{convert::TryInto, ffi::c_void};
use num_derive::FromPrimitive;
use os_units::{Bytes, NumOfPages};
use x86_64::{structures::paging::Size4KiB, PhysAddr, VirtAddr};

/// # Safety
///
/// This function is unsafe because reading a value from I/O port may have side effects which
/// violate memory safety.
#[must_use]
pub unsafe fn inb(port: u16) -> u8 {
    general_syscall(Ty::Inb, port.into(), 0, 0)
        .try_into()
        .unwrap()
}

/// # Safety
///
/// This function is unsafe because reading a value from I/O port may have side effects which violate memory safety.
#[must_use]
pub unsafe fn inl(port: u16) -> u32 {
    general_syscall(Ty::Inl, port.into(), 0, 0)
        .try_into()
        .unwrap_or_else(|_| {
            unreachable!("Inl system call returns a value which is out of the ramge of `u32`.")
        })
}

/// # Safety
///
/// This function is unsafe because writing a value from I/O port may have side effects which
/// violate memory safety.
pub unsafe fn outb(port: u16, value: u8) {
    general_syscall(Ty::Outb, port.into(), value.into(), 0);
}

/// # Safety
///
/// This function is unsafe because writing a value via I/O port may have side effects
/// which violate memory safety.
pub unsafe fn outl(port: u16, value: u32) {
    general_syscall(Ty::Outl, port.into(), value.into(), 0);
}

pub fn halt() {
    // SAFETY: This operation is safe as it does not touch any unsafe things.
    unsafe { general_syscall(Ty::Halt, 0, 0, 0) };
}

pub fn disable_interrupt() {
    // SAFETY: This operation is safe as it does not touch any unsafe things.
    unsafe { general_syscall(Ty::DisableInterrupt, 0, 0, 0) };
}

pub fn enable_interrupt() {
    // SAFETY: This operation is safe as it does not touch any unsafe things.
    unsafe { general_syscall(Ty::EnableInterrupt, 0, 0, 0) };
}

pub fn enable_interrupt_and_halt() {
    // SAFETY: This operation is safe as it does not touch any unsafe things.
    unsafe { general_syscall(Ty::EnableInterruptAndHalt, 0, 0, 0) };
}

#[must_use]
pub fn allocate_pages(pages: NumOfPages<Size4KiB>) -> VirtAddr {
    // SAFETY: This operation is safe as the arguments are propertly passed.
    VirtAddr::new(unsafe {
        general_syscall(
            Ty::AllocatePages,
            pages
                .as_usize()
                .try_into()
                .unwrap_or_else(|_| unreachable!("On x86_64 architecture, `u64` == `usize`.")),
            0,
            0,
        )
    })
}

pub fn deallocate_pages(virt: VirtAddr, pages: NumOfPages<Size4KiB>) {
    // SAFETY: This operation is safe as the all arguments are propertly passed.
    unsafe {
        general_syscall(
            Ty::DeallocatePages,
            virt.as_u64(),
            pages
                .as_usize()
                .try_into()
                .unwrap_or_else(|_| unreachable!("On x86_64 architecture, `u64` == `usize`.")),
            0,
        )
    };
}

#[must_use]
pub fn map_pages(start: PhysAddr, bytes: Bytes) -> VirtAddr {
    // SAFETY: This operation is safe as the all arguments are propertly passed.
    VirtAddr::new(unsafe {
        general_syscall(
            Ty::MapPages,
            start.as_u64(),
            bytes
                .as_usize()
                .try_into()
                .unwrap_or_else(|_| unreachable!("On x86_64 architecture, `u64` == `usize`.")),
            0,
        )
    })
}

pub fn unmap_pages(start: VirtAddr, bytes: Bytes) {
    unsafe {
        general_syscall(
            Ty::UnmapPages,
            start.as_u64(),
            bytes
                .as_usize()
                .try_into()
                .unwrap_or_else(|_| unreachable!("On x86_64 architecture, `usize` == `u64`.")),
            0,
        );
    }
}

#[must_use]
pub fn getpid() -> i32 {
    // SAFETY: The system call type is correct, and the remaining arguments are not used.
    unsafe {
        general_syscall(Ty::GetPid, 0, 0, 0)
            .try_into()
            .unwrap_or_else(|_| unreachable!("PID is out of `i32` range."))
    }
}

pub fn exit() -> ! {
    let ty = Ty::Exit as u64;
    unsafe { asm!("syscall", in("rax") ty, options(noreturn)) }
}

/// This method will return a null address if the address is not mapped.
#[must_use]
pub fn translate_address(a: VirtAddr) -> PhysAddr {
    // SAFETY: Parameters are passed properly.
    PhysAddr::new(unsafe { general_syscall(Ty::TranslateAddress, a.as_u64(), 0, 0) })
}

/// # Safety
///
/// `buf` must be valid.
#[must_use]
pub unsafe fn write(fildes: i32, buf: *const c_void, nbyte: u32) -> i32 {
    // SAFETY: The arguments are fulfilled properly.
    general_syscall(
        Ty::Write,
        fildes.try_into().unwrap(),
        buf as _,
        nbyte.into(),
    )
    .try_into()
    .unwrap()
}

#[must_use]
pub fn notify_exists() -> bool {
    // SAFETY: Arguments are passed properly.
    unsafe { general_syscall(Ty::NotifyExists, 0, 0, 0) != 0 }
}

pub fn notify_on_interrupt(vec: usize, pid: i32) {
    // SAFETY: The arguments are passed correctly.
    unsafe {
        general_syscall(
            Ty::NotifyOnInterrupt,
            vec.try_into().unwrap(),
            pid.try_into().unwrap(),
            0,
        );
    }
}

/// SAFETY: This function is unsafe if arguments are invalid.
#[allow(clippy::too_many_arguments)]
unsafe fn general_syscall(ty: Ty, a1: u64, a2: u64, a3: u64) -> u64 {
    let ty = ty as u64;
    let r: u64;
    asm!("syscall",
        inout("rax") ty => r, inout("rdi") a1 => _, inout("rsi") a2 => _, inout("rdx") a3 => _,
        out("rcx") _, out("r8") _, out("r9") _, out("r10") _, out("r11") _,);
    r
}

#[derive(FromPrimitive)]
pub enum Ty {
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
    GetPid,
    Exit,
    TranslateAddress,
    Write,
    NotifyExists,
    NotifyOnInterrupt,
}
