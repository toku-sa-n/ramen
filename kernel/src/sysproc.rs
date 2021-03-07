// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;
use message::Message;
use num_traits::FromPrimitive;
use x86_64::{instructions::port::PortReadOnly, structures::port::PortRead};

const PID: i32 = 0;

pub(super) fn main() {
    ensure_pid_is_correct();
    main_loop();
}

fn main_loop() {
    loop {
        main_loop_iteration();
    }
}

fn main_loop_iteration() {
    let mut m = Message::default();

    syscalls::receive_from_any(&mut m);
    handle_message(m);
}

fn ensure_pid_is_correct() {
    let i = syscalls::getpid();
    assert_eq!(i, PID, "Wrong PID for the system process: {:?}", i);
}

fn handle_message(m: Message) {
    let t = FromPrimitive::from_u64(m.body.0);
    if let Some(t) = t {
        select_system_calls(m, t);
    } else {
        warn!("Unrecognized message: {:?}", m)
    }
}

fn select_system_calls(m: Message, t: syscalls::Ty) {
    match t {
        syscalls::Ty::Inb => unsafe { reply_inb(m) },
        syscalls::Ty::Inl => unsafe { reply_inl(m) },
        _ => todo!(),
    }
}

unsafe fn reply_inb(m: Message) {
    let r = inb(m);
    reply_with_result(m, r.into());
}

unsafe fn reply_inl(m: Message) {
    let r = inl(m);
    reply_with_result(m, r.into());
}

fn reply_with_result(received: Message, result: u64) {
    let h = message::Header::new(PID);
    let b = message::Body(result, 0, 0, 0, 0);

    let reply = Message::new(h, b);
    let to = received.header.sender;

    syscalls::send(reply, to);
}

/// # Safety
///
/// `m.body.1` must be the valid port number.
unsafe fn inb(m: Message) -> u8 {
    read_port(m)
}

unsafe fn inl(m: Message) -> u32 {
    read_port(m)
}

unsafe fn read_port<T: PortRead>(m: Message) -> T {
    let p = m.body.1;
    let mut p = PortReadOnly::new(p.try_into().unwrap());

    p.read()
}