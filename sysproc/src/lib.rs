// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]

#[macro_use]
extern crate log;

mod port;

use message::Message;
use num_traits::FromPrimitive;

const PID: i32 = 0;

pub fn main() {
    ensure_pid_is_correct();
    main_loop();
}

fn main_loop() {
    loop {
        main_loop_iteration();
    }
}

fn main_loop_iteration() {
    let m = syscalls::receive_from_any();
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
        syscalls::Ty::Outb => unsafe { reply_outb(m) },
        syscalls::Ty::Outl => unsafe { reply_outl(m) },
        _ => todo!(),
    }
}

unsafe fn reply_inb(m: Message) {
    let r = port::inb(m);
    reply_with_result(m, r.into());
}

unsafe fn reply_inl(m: Message) {
    let r = port::inl(m);
    reply_with_result(m, r.into());
}

unsafe fn reply_outb(m: Message) {
    port::outb(m);
    reply_without_contents(m);
}

unsafe fn reply_outl(m: Message) {
    port::outl(m);
    reply_without_contents(m);
}

fn reply_with_result(received: Message, result: u64) {
    let h = message::Header::new(PID);
    let b = message::Body(result, 0, 0, 0, 0);

    let reply = Message::new(h, b);
    let to = received.header.sender;

    syscalls::send(reply, to);
}

fn reply_without_contents(received: Message) {
    let h = message::Header::new(PID);
    let b = message::Body::default();

    let reply = Message::new(h, b);
    let to = received.header.sender;

    syscalls::send(reply, to);
}
