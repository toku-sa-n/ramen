#![no_std]
#![feature(lang_items, start, asm)]
#![no_main]

extern crate rlibc;

#[macro_use]
extern crate log;

extern crate uefi;
extern crate uefi_services;

#[macro_use]
#[allow(unused_imports)]
extern crate debug;

mod exit;
mod fs;
mod gop;
mod init;
mod mem;

use core::ptr;
use core::slice;
use exit::BootInfo;
use uefi::prelude::{Boot, Handle, SystemTable};
use uefi::table::boot;
use uefi::table::boot::MemoryType;
use uefi::ResultExt;

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
    let memory_map_buf = system_table
        .boot_services()
        .allocate_pool(
            MemoryType::LOADER_DATA,
            system_table.boot_services().memory_map_size(),
        )
        .expect_success("Failed to allocate memory for memory map")
        as *mut boot::MemoryDescriptor;

    let buf_for_exiting = system_table
        .boot_services()
        .allocate_pool(
            MemoryType::LOADER_DATA,
            system_table.boot_services().memory_map_size() * 2,
        )
        .expect_success("Failed to allocate memory to exit boot services");
    let buf_for_exiting = unsafe {
        slice::from_raw_parts_mut(
            buf_for_exiting,
            system_table.boot_services().memory_map_size() * 2,
        )
    };

    let (_, descriptors_iter) = system_table
        .exit_boot_services(image, buf_for_exiting)
        .expect("Failed to exit boot services")
        .unwrap();

    let mut num_descriptors = 0;
    for (index, descriptor) in descriptors_iter.enumerate() {
        unsafe {
            ptr::write(memory_map_buf.offset(index as isize), *descriptor);
        }

        num_descriptors += 1;
    }

    unsafe { slice::from_raw_parts_mut(memory_map_buf, num_descriptors) }
}
