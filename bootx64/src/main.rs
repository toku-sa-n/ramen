#![no_std]
#![feature(lang_items, start)]
#![no_main]

#[macro_use]
extern crate log;

extern crate uefi;
extern crate uefi_services;

use core::ptr;
use uefi::prelude::{Boot, Handle, Status, SystemTable};
use uefi::proto::console::gop;
use uefi::proto::loaded_image;
use uefi::proto::media::file;
use uefi::proto::media::fs;
use uefi::table::boot::MemoryType;
use uefi::table::boot::SearchType;
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

fn get_buf_len_for_locate_handler(
    system_table: &SystemTable<Boot>,
    search_type: SearchType,
) -> usize {
    // To get the length of buffer, this function should be called with None.
    // See: https://docs.rs/uefi/0.4.7/uefi/table/boot/struct.BootServices.html#method.locate_handle
    system_table
        .boot_services()
        .locate_handle(search_type, None)
        .expect_success("Failed to get buffer length for locate_handler.")
}

fn malloc<T: Sized>(system_table: &SystemTable<Boot>, num: usize) -> uefi::Result<*mut T> {
    let buffer = system_table
        .boot_services()
        .allocate_pool(MemoryType::LOADER_DATA, num * core::mem::size_of::<T>());

    match buffer {
        Err(e) => Err(e),
        Ok(buf) => Ok(buf.map(|x| x as *mut T)),
    }
}

fn get_gop<'a>(system_table: &'a SystemTable<Boot>) -> gop::GraphicsOutput<'a> {
    let search_type = SearchType::from_proto::<gop::GraphicsOutput>();

    let buf_length = get_buf_len_for_locate_handler(system_table, search_type);

    let buf = malloc::<Handle>(system_table, buf_length)
        .expect_success("Failed to allocate buffer for gop.");

    system_table
        .boot_services()
        .locate_handle(
            search_type,
            Some(unsafe { core::slice::from_raw_parts_mut(buf, buf_length) }),
        )
        .expect_success("Failed to locate gop's handle.");

    let gop_buf = unsafe { ptr::read(buf) };

    let gop = system_table
        .boot_services()
        .handle_protocol::<gop::GraphicsOutput>(gop_buf)
        .expect_success("Failed to get gop.");

    unsafe { ptr::read(gop.get()) }
}

fn open_root_dir(image: &Handle, system_table: &SystemTable<Boot>) -> file::Directory {
    let loaded_image = system_table
        .boot_services()
        .handle_protocol::<loaded_image::LoadedImage>(*image)
        .expect_success("Failed to load image");

    let loaded_image = unsafe { &*loaded_image.get() };

    let simple_file_system = system_table
        .boot_services()
        .handle_protocol::<fs::SimpleFileSystem>(loaded_image.device())
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

fn init_gop(image: &Handle, system_table: &SystemTable<Boot>) -> () {
    get_gop(system_table);
}

#[start]
#[no_mangle]
pub fn efi_main(image: Handle, system_table: SystemTable<Boot>) -> Status {
    initialize(&system_table);
    open_root_dir(&image, &system_table);
    info!("Opened volume");
    init_gop(&image, &system_table);
    loop {}
}
