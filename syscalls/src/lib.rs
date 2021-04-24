// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![feature(asm)]
#![allow(clippy::missing_panics_doc)]

use core::{convert::TryInto, ffi::c_void, panic::PanicInfo};
use message::Message;
use num_derive::FromPrimitive;
use os_units::{Bytes, NumOfPages};
use x86_64::{structures::paging::Size4KiB, PhysAddr, VirtAddr};

/// # Safety
///
/// This function is unsafe because reading a value from I/O port may have side effects which
/// violate memory safety.
#[must_use]
pub unsafe fn inb(port: u16) -> u8 {
    let body = message::Body(Ty::Inb as u64, port.into(), 0, 0, 0);
    let header = message::Header::new(0);
    let m = Message::new(header, body);

    send(m, 0);

    let reply = receive_from_any();

    reply.body.0.try_into().unwrap()
}

/// # Safety
///
/// This function is unsafe because reading a value from I/O port may have side effects which violate memory safety.
#[must_use]
pub unsafe fn inl(port: u16) -> u32 {
    let body = message::Body(Ty::Inl as u64, port.into(), 0, 0, 0);
    let header = message::Header::new(0);
    let m = Message::new(header, body);

    send(m, 0);

    let reply = receive_from_any();

    reply.body.0.try_into().unwrap()
}

/// # Safety
///
/// This function is unsafe because writing a value from I/O port may have side effects which
/// violate memory safety.
pub unsafe fn outb(port: u16, value: u8) {
    let body = message::Body(Ty::Outb as u64, port.into(), value.into(), 0, 0);
    let header = message::Header::new(0);
    let m = Message::new(header, body);

    send(m, 0);

    receive_ack();
}

/// # Safety
///
/// This function is unsafe because writing a value via I/O port may have side effects
/// which violate memory safety.
pub unsafe fn outl(port: u16, value: u32) {
    let body = message::Body(Ty::Outl as u64, port.into(), value.into(), 0, 0);
    let header = message::Header::new(0);
    let m = Message::new(header, body);

    send(m, 0);

    receive_ack();
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
    let body = message::Body(Ty::GetPid as u64, 0, 0, 0, 0);
    let header = message::Header::default();
    let m = Message::new(header, body);

    send(m, 1);

    let reply = receive_from_any();

    reply.body.0.try_into().unwrap()
}

pub fn exit() -> ! {
    let ty = Ty::Exit as u64;
    unsafe { asm!("int 0x80", in("rax") ty, options(noreturn)) }
}

/// This method will return a null address if the address is not mapped.
#[must_use]
pub fn translate_address(a: VirtAddr) -> PhysAddr {
    // SAFETY: Parameters are passed properly.
    PhysAddr::new(unsafe { general_syscall(Ty::TranslateAddress, a.as_u64(), 0, 0) })
}

pub fn send(m: Message, to: i32) {
    let ty = Ty::Send as u64;
    let a1 = &m;
    let a1: *const Message = a1;
    let a1: u64 = a1 as _;
    let a2 = to;
    let a3 = 0;
    unsafe {
        asm!("int 0x81",
        inout("rax") ty => _, inout("rdi") a1 => _, inout("rsi") a2 => _, inout("rdx") a3 => _,
        out("rcx") _, out("r8") _, out("r9") _, out("r10") _, out("r11") _,);
    }
}

#[must_use]
pub fn receive_from_any() -> Message {
    let mut m = Message::default();

    let ty = Ty::Receive as u64;
    let a1 = &mut m;
    let a1: *mut Message = a1;
    let a1: u64 = a1 as _;
    let a2 = 0;
    let a3 = 0;

    unsafe {
        asm!("int 0x81",
        inout("rax") ty => _, inout("rdi") a1 => _, inout("rsi") a2 => _, inout("rdx") a3 => _,
        out("rcx") _, out("r8") _, out("r9") _, out("r10") _, out("r11") _,);
    }

    m
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

pub fn panic(info: &PanicInfo<'_>) -> ! {
    let info: *const PanicInfo<'_> = info;
    unsafe { general_syscall(Ty::Panic, info as _, 0, 0) };
    unreachable!("The `panic` system call should not return.");
}

/// SAFETY: This function is unsafe if arguments are invalid.
#[allow(clippy::too_many_arguments)]
unsafe fn general_syscall(ty: Ty, a1: u64, a2: u64, a3: u64) -> u64 {
    let ty = ty as u64;
    let r: u64;
    asm!("int 0x80",
        inout("rax") ty => r, inout("rdi") a1 => _, inout("rsi") a2 => _, inout("rdx") a3 => _,
        out("rcx") _, out("r8") _, out("r9") _, out("r10") _, out("r11") _,);
    r
}

fn receive_ack() {
    let _ = receive_from_any();
}

#[derive(Copy, Clone, FromPrimitive, Debug)]
pub enum Ty {
    Inb,
    Outb,
    Inl,
    Outl,
    AllocatePages,
    DeallocatePages,
    MapPages,
    UnmapPages,
    GetPid,
    Exit,
    TranslateAddress,
    Write,
    Send,
    Receive,
    Panic,
}
