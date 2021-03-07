// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;
use message::Message;
use x86_64::{
    instructions::port::{PortReadOnly, PortWriteOnly},
    structures::port::PortRead,
};

pub(super) unsafe fn inb(m: Message) -> u8 {
    read_from_port(m)
}

pub(super) unsafe fn inl(m: Message) -> u32 {
    read_from_port(m)
}

pub(super) unsafe fn outb(m: Message) {
    let message::Body(_, p, v, ..) = m.body;
    let mut p = PortWriteOnly::<u8>::new(p.try_into().unwrap());

    p.write(v.try_into().unwrap());
}

unsafe fn read_from_port<T: PortRead>(m: Message) -> T {
    let p = m.body.1;
    let mut p = PortReadOnly::new(p.try_into().unwrap());

    p.read()
}
