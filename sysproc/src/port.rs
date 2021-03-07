// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;
use message::Message;
use x86_64::{instructions::port::PortReadOnly, structures::port::PortRead};

pub(super) unsafe fn inb(m: Message) -> u8 {
    read_port(m)
}

pub(super) unsafe fn inl(m: Message) -> u32 {
    read_port(m)
}

unsafe fn read_port<T: PortRead>(m: Message) -> T {
    let p = m.body.1;
    let mut p = PortReadOnly::new(p.try_into().unwrap());

    p.read()
}
