#![no_std]
#![feature(lang_items, start, asm)]
#![no_main]

#[macro_use]
extern crate log;

extern crate uefi;
extern crate uefi_services;

#[macro_use]
mod debug;

mod exit;
mod fs;
mod gop;
mod memory;

use core::slice;
use exit::BootInfo;
use uefi::prelude::{Boot, Handle, SystemTable};
use uefi::table::boot;
use uefi::ResultExt;

fn reset_console(system_table: &SystemTable<Boot>) -> () {
    system_table
        .stdout()
        .reset(false)
        .expect_success("Failed to reset stdout");
}

/// Initialize uefi-rs services. This includes initialization of GlobalAlloc, which enables us to
/// use Collections defined in alloc module, such as Vec and LinkedList.
fn initialize_uefi_utilities(system_table: &SystemTable<Boot>) -> () {
    uefi_services::init(&system_table).expect_success("Failed to initialize_uefi_utilities");
}

fn initialize(system_table: &SystemTable<Boot>) -> () {
    initialize_uefi_utilities(&system_table);
    reset_console(&system_table);
    info!("Hello World!");
}

fn terminate_boot_services<'a>(
    image: Handle,
    system_table: SystemTable<Boot>,
) -> &'a mut [boot::MemoryDescriptor] {
    let (memory_map, memory_map_size) = memory::generate_map(&system_table);

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
#[start]
#[no_mangle]
pub fn efi_main(image: Handle, system_table: SystemTable<Boot>) -> ! {
    initialize(&system_table);

    let vram_info = gop::init(&system_table);

    fs::place_kernel(&system_table);
    let mem_map = terminate_boot_services(image, system_table);

    exit::bootx64(mem_map, BootInfo::new(vram_info));
}
