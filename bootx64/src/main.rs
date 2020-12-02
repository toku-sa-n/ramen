// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![feature(start, asm)]
#![no_main]
#![deny(clippy::pedantic)]
#![deny(clippy::all)]

#[macro_use]
#[allow(unused_imports)]
extern crate common;

#[macro_use]
extern crate log;

extern crate x86_64;

mod exit;
mod fs;
mod gop;
mod mem;

use common::{
    constant::{INITRD_NAME, KERNEL_NAME},
    kernelboot,
    mem::reserved,
};
use core::{convert::TryInto, ptr, ptr::NonNull, slice};
use mem::{paging, stack};
use uefi::{
    prelude::{Boot, Handle, SystemTable},
    table::{boot, boot::MemoryType},
    ResultExt,
};

#[start]
#[no_mangle]
pub fn efi_main(image: Handle, system_table: SystemTable<Boot>) -> ! {
    init_libs(&system_table);

    let vram_info = gop::init(system_table.boot_services());

    let (phys_kernel_addr, bytes_kernel) = fs::deploy(system_table.boot_services(), KERNEL_NAME);
    let (entry_addr, actual_mem_size) =
        fs::fetch_entry_address_and_memory_size(phys_kernel_addr, bytes_kernel);

    let (initrd_addr, bytes_initrd) = fs::deploy(system_table.boot_services(), INITRD_NAME);

    let stack_addr = stack::allocate(system_table.boot_services());
    let reserved_regions = reserved::Map::new(
        &reserved::KernelPhysRange::new(phys_kernel_addr, actual_mem_size),
        stack_addr,
        &vram_info,
    );
    let mem_map = terminate_boot_services(image, system_table);

    let mut boot_info = kernelboot::Info::new(entry_addr, vram_info, mem_map);

    paging::init(&mut boot_info, &reserved_regions);
    exit::bootx64(boot_info);
}

fn init_libs(system_table: &SystemTable<Boot>) {
    initialize_uefi_utilities(&system_table);
    reset_console(&system_table);
    info!("Hello World!");
}

fn initialize_uefi_utilities(system_table: &SystemTable<Boot>) {
    uefi_services::init(system_table).expect_success("Failed to initialize_uefi_utilities");
}

fn reset_console(system_table: &SystemTable<Boot>) {
    system_table
        .stdout()
        .reset(false)
        .expect_success("Failed to reset stdout");
}

fn terminate_boot_services(image: Handle, system_table: SystemTable<Boot>) -> common::mem::Map {
    info!("Goodbye, boot services...");
    let memory_map_buf = NonNull::new(
        system_table
            .boot_services()
            .allocate_pool(
                MemoryType::LOADER_DATA,
                system_table.boot_services().memory_map_size(),
            )
            .expect_success("Failed to allocate memory for memory map"),
    )
    .unwrap()
    .cast::<boot::MemoryDescriptor>();

    let buf = allocate_buf_for_exiting(system_table.boot_services());
    let (_, mut descriptors_iter) = system_table
        .exit_boot_services(image, buf)
        .expect("Failed to exit boot services")
        .unwrap();

    let num_descriptors = descriptors_iter.len();
    let memory_map_buf = write_descriptors_on_buf(memory_map_buf, &mut descriptors_iter);
    common::mem::Map::new(memory_map_buf, num_descriptors)
}

fn allocate_buf_for_exiting(bs: &boot::BootServices) -> &'static mut [u8] {
    // Allocate extra spaces because of paddings.
    let sz = bs.memory_map_size() * 2;
    let buf_for_exiting = bs
        .allocate_pool(MemoryType::LOADER_DATA, sz)
        .expect_success("Failed to allocate memory to exit boot services");
    unsafe { slice::from_raw_parts_mut(buf_for_exiting, sz) }
}

fn write_descriptors_on_buf(
    buf: NonNull<boot::MemoryDescriptor>,
    iter: &mut dyn ExactSizeIterator<Item = &boot::MemoryDescriptor>,
) -> NonNull<boot::MemoryDescriptor> {
    for (index, descriptor) in iter.enumerate() {
        unsafe {
            ptr::write(buf.as_ptr().offset(index.try_into().unwrap()), *descriptor);
        }
    }

    buf
}
