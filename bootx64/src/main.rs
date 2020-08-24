#![no_std]
#![feature(start, asm)]
#![no_main]

extern crate rlibc;

#[macro_use]
extern crate log;

extern crate elf_rs;
extern crate uefi;
extern crate uefi_services;

#[macro_use]
#[allow(unused_imports)]
extern crate debug;

extern crate common_items;

extern crate x86_64;

mod exit;
mod fs;
mod gop;
mod mem;

use core::ptr;
use core::slice;
use fs::kernel;
use mem::stack;
use uefi::prelude::{Boot, Handle, SystemTable};
use uefi::table::boot;
use uefi::table::boot::MemoryType;
use uefi::ResultExt;

#[start]
#[no_mangle]
pub fn efi_main(image: Handle, system_table: SystemTable<Boot>) -> ! {
    uefi(&system_table);

    let vram_info = gop::init(system_table.boot_services());

    let (phys_kernel_addr, bytes_kernel) = kernel::deploy(system_table.boot_services());
    let (entry_addr, actual_memory_size) =
        kernel::fetch_entry_address_and_memory_size(phys_kernel_addr, bytes_kernel);

    info!("Entry point: {:?}", entry_addr);
    info!("Memory size: {:X?}", actual_memory_size.as_usize());

    let stack_addr = stack::allocate(system_table.boot_services());
    let mem_map = terminate_boot_services(image, system_table);

    let mem_map_info = common_items::MemMapInfo::new_from_slice(mem_map);

    exit::bootx64(
        mem_map,
        common_items::BootInfo::new(vram_info, mem_map_info),
        entry_addr,
        phys_kernel_addr,
        actual_memory_size,
        stack_addr,
    );
}

fn uefi(system_table: &SystemTable<Boot>) -> () {
    initialize_uefi_utilities(&system_table);
    reset_console(&system_table);
    info!("Hello World!");
}

/// Initialize uefi-rs services. This includes initialization of GlobalAlloc, which enables us to
/// use Collections defined in alloc module, such as Vec and LinkedList.
fn initialize_uefi_utilities(system_table: &SystemTable<Boot>) -> () {
    uefi_services::init(system_table).expect_success("Failed to initialize_uefi_utilities");
}

fn reset_console(system_table: &SystemTable<Boot>) -> () {
    system_table
        .stdout()
        .reset(false)
        .expect_success("Failed to reset stdout");
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
