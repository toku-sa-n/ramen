#![no_std]
#![feature(lang_items, start, asm)]
#![no_main]

#[macro_use]
extern crate log;

extern crate uefi;
extern crate uefi_services;

mod fs;
mod gop;
mod memory;

use uefi::prelude::{Boot, Handle, Status, SystemTable};
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

fn terminate_boot_services(image: Handle, system_table: SystemTable<Boot>) -> () {
    let memory_map = memory::generate_map(&system_table);

    let memory_map_size = system_table.boot_services().memory_map_size() * 2;

    system_table
        .exit_boot_services(image, unsafe {
            core::slice::from_raw_parts_mut(memory_map, memory_map_size)
        })
        .expect("Failed to exit boot services")
        .unwrap();
}

#[start]
#[no_mangle]
pub fn efi_main(image: Handle, system_table: SystemTable<Boot>) -> Status {
    initialize(&system_table);
    gop::init(&system_table);
    info!("GOP set.");
    fs::place_binary_files(&system_table);
    terminate_boot_services(image, system_table);

    unsafe {
        asm!("jmp rdi",in("rdi") 0x8000 );
    }

    loop {}
}
