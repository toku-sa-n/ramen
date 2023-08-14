// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![feature(naked_functions)]
#![deny(clippy::pedantic)]
#![deny(clippy::all)]

pub mod edid;
pub mod exit;
pub mod fs;
pub mod gop;
pub mod init;
pub mod jump;
pub mod mem;
pub mod rsdp;

extern crate alloc as _;

pub use init::init;
