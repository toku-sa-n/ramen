#![no_std]
#![feature(lang_items, start, asm)]
#![no_main]

#[macro_use]
extern crate log;

extern crate uefi;
extern crate uefi_services;

#[macro_use]
mod debug;

mod fs;
mod gop;
mod memory;

use core::slice;
use uefi::prelude::{Boot, Handle, Status, SystemTable};
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

fn disable_interruption() -> () {
    // Use `nop` because some machines go wrong when continuously doing `out`.
    unsafe {
        asm!(
            "mov al,0xff
            out 0x21,al
            nop
            out 0xa1,al
            cli"
        );
    }
}

fn jump_to_kernel() -> () {
    unsafe {
        asm!("jmp rdi",in("rdi") 0x8000 );
    }
}

#[start]
#[no_mangle]
pub fn efi_main(image: Handle, system_table: SystemTable<Boot>) -> Status {
    initialize(&system_table);
    gop::init(&system_table);
    info!("GOP set.");
    fs::place_binary_files(&system_table);
    terminate_boot_services(image, system_table);

    disable_interruption();
    jump_to_kernel();

    loop {}
}
