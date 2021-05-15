// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![allow(clippy::too_many_arguments)] // A workaround for the clippy's wrong warning.
#![cfg_attr(feature = "heap", feature(alloc_error_handler))]

pub mod io;
pub mod mem;

#[cfg(feature = "heap")]
extern crate alloc;

#[cfg(feature = "heap")]
pub fn init() {
    io::init();
    mem::heap::init();
}

#[cfg(not(feature = "heap"))]
pub fn init() {
    io::init();
}

#[panic_handler]
fn panic(i: &core::panic::PanicInfo<'_>) -> ! {
    syscalls::panic(i);
}
