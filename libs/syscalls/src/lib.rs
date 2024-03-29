// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![allow(clippy::missing_panics_doc)]
#![deny(unsafe_op_in_unsafe_fn)]
#![feature(naked_functions)]

use {
    core::{arch::asm, convert::TryInto, ffi::c_void, panic::PanicInfo},
    message::Message,
    num_derive::FromPrimitive,
    os_units::{Bytes, NumOfPages},
    x86_64::{structures::paging::Size4KiB, PhysAddr, VirtAddr},
};

/// # Safety
///
/// This function is unsafe because reading a value from I/O port may have side effects which
/// violate memory safety.
#[must_use]
pub unsafe fn inb(port: u16) -> u8 {
    let body = message::Body(Ty::Inb as u64, port.into(), 0, 0, 0);
    let header = message::Header::new(0);
    let m = Message::new(header, body);

    send(m, 2);

    let reply = receive_from(2);

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

    send(m, 2);

    let reply = receive_from(2);

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

    send(m, 2);

    receive_ack(2);
}

/// # Safety
///
/// This function is unsafe because writing a value via I/O port may have side effects
/// which violate memory safety.
pub unsafe fn outl(port: u16, value: u32) {
    let body = message::Body(Ty::Outl as u64, port.into(), value.into(), 0, 0);
    let header = message::Header::new(0);
    let m = Message::new(header, body);

    send(m, 2);

    receive_ack(2);
}

#[must_use]
pub fn allocate_pages(pages: NumOfPages<Size4KiB>) -> VirtAddr {
    // SAFETY: This operation is safe as the arguments are propertly passed.
    VirtAddr::new(general_syscall(
        Ty::AllocatePages,
        pages
            .as_usize()
            .try_into()
            .unwrap_or_else(|_| unreachable!("On x86_64 architecture, `u64` == `usize`.")),
        0,
        0,
    ))
}

pub fn deallocate_pages(virt: VirtAddr, pages: NumOfPages<Size4KiB>) {
    // SAFETY: This operation is safe as the all arguments are propertly passed.
    general_syscall(
        Ty::DeallocatePages,
        virt.as_u64(),
        pages
            .as_usize()
            .try_into()
            .unwrap_or_else(|_| unreachable!("On x86_64 architecture, `u64` == `usize`.")),
        0,
    );
}

#[must_use]
pub fn map_pages(start: PhysAddr, bytes: Bytes) -> VirtAddr {
    // SAFETY: This operation is safe as the all arguments are propertly passed.
    VirtAddr::new(general_syscall(
        Ty::MapPages,
        start.as_u64(),
        bytes
            .as_usize()
            .try_into()
            .unwrap_or_else(|_| unreachable!("On x86_64 architecture, `u64` == `usize`.")),
        0,
    ))
}

pub fn unmap_pages(start: VirtAddr, bytes: Bytes) {
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

#[must_use]
pub fn getpid() -> i32 {
    let body = message::Body(Ty::GetPid as u64, 0, 0, 0, 0);
    let header = message::Header::default();
    let m = Message::new(header, body);

    send(m, 1);

    let reply = receive_from(1);

    reply.body.0.try_into().unwrap()
}

/// This method will return a null address if the address is not mapped.
#[must_use]
pub fn translate_address(a: VirtAddr) -> PhysAddr {
    // SAFETY: Parameters are passed properly.
    PhysAddr::new(general_syscall(Ty::TranslateAddress, a.as_u64(), 0, 0))
}

pub fn send(m: Message, to: i32) {
    let ty = Ty::Send;
    let a1 = &m;
    let a1: *const Message = a1;
    let a1: u64 = a1 as _;
    let a2: u64 = to.try_into().unwrap();
    let a3 = 0;

    message_syscall(ty, a1, a2, a3);
}

#[must_use]
pub fn receive_from_any() -> Message {
    let mut m = Message::default();

    let ty = Ty::ReceiveFromAny;
    let a1 = &mut m;
    let a1: *mut Message = a1;
    let a1: u64 = a1 as _;
    let a2 = 0;
    let a3 = 0;

    message_syscall(ty, a1, a2, a3);

    m
}

#[must_use]
pub fn receive_from(from: i32) -> Message {
    let mut m = Message::default();

    let m_ptr: *mut Message = &mut m;
    let m_ptr: u64 = m_ptr as _;

    message_syscall(Ty::ReceiveFrom, m_ptr, from.try_into().unwrap(), 0);

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
    general_syscall(Ty::Panic, info as _, 0, 0);
    unreachable!("The `panic` system call should not return.");
}

fn receive_ack(from: i32) {
    let _ = receive_from(from);
}

#[derive(Copy, Clone, FromPrimitive, Debug)]
#[repr(u64)]
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
    TranslateAddress,
    Write,
    Send,
    ReceiveFromAny,
    ReceiveFrom,
    Panic,
}

#[naked]
#[allow(clippy::too_many_lines)]
extern "C" fn general_syscall(ty: Ty, a1: u64, a2: u64, a3: u64) -> u64 {
    unsafe {
        asm!(
            "

    push rax
    push rdi
    push rsi
    push rdx
    push rcx
    push r8
    push r9
    push r10
    push r11

    mov rax, rdi
    mov rdi, rsi
    mov rsi, rdx
    mov rdx, rcx
    syscall

    pop r11
    pop r10
    pop r9
    pop r8
    pop rcx
    pop rdx
    pop rsi
    pop rdi
    add rsp, 8  /* RAX is used to return a value. */
    ret
    ",
            options(noreturn)
        );
    }
}

#[naked]
#[allow(clippy::too_many_lines)]
extern "sysv64" fn message_syscall(ty: Ty, a1: u64, a2: u64, a3: u64) {
    unsafe {
        asm!(
            "
    push rax
    push rdi
    push rsi
    push rdx
    push rcx
    push r8
    push r9
    push r10
    push r11

    mov rax, rdi
    mov rdi, rsi
    mov rsi, rdx
    mov rdx, rcx
    syscall

    pop r11
    pop r10
    pop r9
    pop r8
    pop rcx
    pop rdx
    pop rsi
    pop rdi
    pop rax
    ret
    ",
            options(noreturn)
        );
    }
}
