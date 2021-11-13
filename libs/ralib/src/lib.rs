// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![allow(clippy::too_many_arguments)] // A workaround for the clippy's wrong warning.
#![feature(alloc_error_handler)]
#![deny(unsafe_op_in_unsafe_fn)]

use core::alloc::Layout;

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

#[alloc_error_handler]
fn alloc_fail(l: Layout) -> ! {
    panic!("Allocation failed: {:?}", l)
}
