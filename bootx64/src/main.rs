// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![feature(start, asm)]
#![no_main]
#![deny(clippy::pedantic)]
#![deny(clippy::all)]

#[macro_use]
#[allow(unused_imports)]
extern crate common;

use {
    bootx64::{
        fs, gop, jump,
        mem::{paging, stack},
        rsdp,
    },
    common::mem::reserved,
    uefi::prelude::{Boot, Handle, SystemTable},
};

#[start]
#[no_mangle]
pub fn efi_main(image: Handle, mut system_table: SystemTable<Boot>) -> ! {
    bootx64::init(&mut system_table);

    let vram_info = gop::init(system_table.boot_services());

    let (phys_kernel_addr, bytes_kernel) = fs::deploy(system_table.boot_services(), "kernel.bin");
    let (phys_initrd_addr, bytes_initrd) = fs::deploy(system_table.boot_services(), "initrd.cpio");
    let (entry_addr, actual_mem_size) =
        fs::fetch_entry_address_and_memory_size(phys_kernel_addr, bytes_kernel);

    let stack_addr = stack::allocate(system_table.boot_services());
    let rsdp = rsdp::get(&system_table);
    let reserved_regions = reserved::Map::new(
        &reserved::PhysRange::new(phys_kernel_addr, actual_mem_size),
        stack_addr,
        &vram_info,
        phys_initrd_addr,
        bytes_initrd,
    );
    let mem_map = bootx64::exit::boot_services(image, system_table);

    let mut boot_info = boot_info::Info::new(entry_addr, vram_info, mem_map, rsdp);

    paging::init(&mut boot_info, &reserved_regions);
    jump::to_kernel(boot_info);
}
