// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![feature(const_fn)]

pub mod boot;
pub mod constant;
pub mod debug;
pub mod mem;
pub mod size;
pub mod vram;

extern crate uefi;
extern crate x86_64;
