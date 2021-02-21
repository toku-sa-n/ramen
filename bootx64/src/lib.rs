// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![feature(asm)]
#![deny(clippy::pedantic)]
#![deny(clippy::all)]

pub mod exit;
pub mod fs;
pub mod gop;
pub mod init;
pub mod jump;
pub mod mem;
pub mod rsdp;

#[macro_use]
extern crate log;
extern crate alloc;

pub use init::init;
