#![no_std]
#![feature(lang_items, start)]
#![no_main]

#[macro_use]
extern crate log;

extern crate uefi;
extern crate uefi_services;

mod gop;

use uefi::prelude::{Boot, Handle, Status, SystemTable};
use uefi::proto::media::file;
use uefi::proto::media::fs;
use uefi::ResultExt;

fn reset_console(system_table: &SystemTable<Boot>) -> () {
    system_table
        .stdout()
        .reset(false)
        .expect_success("Failed to reset stdout");
}

fn initialize_uefi_utilities(system_table: &SystemTable<Boot>) -> () {
    uefi_services::init(&system_table).expect_success("Failed to initialize_uefi_utilities");
}

fn open_root_dir(system_table: &SystemTable<Boot>) -> file::Directory {
    let simple_file_system = system_table
        .boot_services()
        .locate_protocol::<fs::SimpleFileSystem>()
        .expect_success("Failed to prepare simple file system.");

    let simple_file_system = unsafe { &mut *simple_file_system.get() };

    simple_file_system
        .open_volume()
        .expect_success("Failed to open volume.")
}

fn initialize(system_table: &SystemTable<Boot>) -> () {
    initialize_uefi_utilities(&system_table);
    reset_console(&system_table);
    info!("Hello World!");
}

#[start]
#[no_mangle]
pub fn efi_main(image: Handle, system_table: SystemTable<Boot>) -> Status {
    initialize(&system_table);
    open_root_dir(&system_table);
    info!("Opened volume");
    gop::init_gop(&system_table);
    info!("GOP set.");
    loop {}
}
