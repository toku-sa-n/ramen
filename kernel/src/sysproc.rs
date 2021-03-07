// SPDX-License-Identifier: GPL-3.0-or-later

use crate::process::ipc;
use core::convert::TryInto;
use message::Message;
use num_traits::FromPrimitive;
use x86_64::{instructions::port::PortReadOnly, VirtAddr};

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
    if let Some(syscalls::Ty::Inb) = t {
        unsafe { reply_inb(m) }
    } else {
        warn!("Unrecognized message: {:?}", m)
    }
}

unsafe fn reply_inb(m: Message) {
    let r = inb(m);
    let h = message::Header::new(PID);
    let b = message::Body(r.into(), 0, 0, 0, 0);

    let reply = Message::new(h, b);
    let reply_addr = &reply as *const Message as u64;
    let reply_addr = VirtAddr::new(reply_addr);

    let to = m.header.sender;

    ipc::send(reply_addr, to);
}

/// # Safety
///
/// `m.body.1` must be the valid port number.
unsafe fn inb(m: Message) -> u8 {
    let p = m.body.1;
    let mut p = PortReadOnly::new(p.try_into().unwrap());

    p.read()
}
