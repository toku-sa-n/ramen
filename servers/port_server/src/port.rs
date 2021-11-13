// SPDX-License-Identifier: GPL-3.0-or-later

use {
    core::convert::{TryFrom, TryInto},
    message::Message,
    x86_64::{
        instructions::port::{PortReadOnly, PortWriteOnly},
        structures::port::{PortRead, PortWrite},
    },
};

pub(super) unsafe fn inb(m: Message) -> u8 {
    unsafe { read_from_port(m) }
}

pub(super) unsafe fn inl(m: Message) -> u32 {
    unsafe { read_from_port(m) }
}

pub(super) unsafe fn outb(m: Message) {
    unsafe {
        write_to_port::<u8>(m);
    }
}

pub(super) unsafe fn outl(m: Message) {
    unsafe {
        write_to_port::<u32>(m);
    }
}

unsafe fn read_from_port<T: PortRead>(m: Message) -> T {
    let p = m.body.1;
    let mut p = PortReadOnly::new(p.try_into().unwrap());

    unsafe { p.read() }
}

unsafe fn write_to_port<T: PortWrite + TryFrom<u64>>(m: Message)
where
    <T as TryFrom<u64>>::Error: core::fmt::Debug,
{
    let message::Body(_, p, v, ..) = m.body;
    let mut p = PortWriteOnly::<T>::new(p.try_into().unwrap());

    unsafe {
        p.write(T::try_from(v).unwrap());
    }
}
