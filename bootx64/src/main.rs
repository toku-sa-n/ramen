#![no_std]
#![feature(lang_items, start, asm)]
#![no_main]

extern crate rlibc;

#[macro_use]
extern crate log;

extern crate uefi;
extern crate uefi_services;

#[macro_use]
mod debug;

mod exit;
mod fs;
mod gop;
mod init;
mod mem;

use core::slice;
use exit::BootInfo;
use mem::map;
use uefi::prelude::{Boot, Handle, SystemTable};
use uefi::table::boot;

#[start]
#[no_mangle]
pub fn efi_main(image: Handle, system_table: SystemTable<Boot>) -> ! {
    init::uefi(&system_table);

    let vram_info = gop::init(&system_table);

    fs::place_kernel(&system_table);
    let mem_map = terminate_boot_services(image, system_table);

    exit::bootx64(mem_map, BootInfo::new(vram_info));
}

fn terminate_boot_services<'a>(
    image: Handle,
    system_table: SystemTable<Boot>,
) -> &'a mut [boot::MemoryDescriptor] {
    let (memory_map, memory_map_size) = map::generate_map(&system_table);

    system_table
        .exit_boot_services(image, unsafe {
            core::slice::from_raw_parts_mut(memory_map, memory_map_size)
        })
        .expect("Failed to exit boot services")
        .unwrap();

    unsafe {
        slice::from_raw_parts_mut::<boot::MemoryDescriptor>(memory_map as *mut _, memory_map_size)
    }
}
