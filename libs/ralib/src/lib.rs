// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![allow(clippy::too_many_arguments)] // A workaround for the clippy's wrong warning.
#![deny(unsafe_op_in_unsafe_fn)]

pub mod io;
pub mod mem;

extern crate alloc;

pub fn init() {
    io::init();
}

#[panic_handler]
fn panic(i: &core::panic::PanicInfo<'_>) -> ! {
    syscalls::panic(i);
}
