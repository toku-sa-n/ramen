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

use core::mem;
use core::ptr;
use core::slice;
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

struct BootInfo {
    _vram_info: gop::VramInfo,
}

impl BootInfo {
    fn new(_vram_info: gop::VramInfo) -> Self {
        Self { _vram_info }
    }
}

const INIT_RSP: usize = 0xffff_ffff_800a_1000 - mem::size_of::<BootInfo>();

fn save_boot_info(boot_info: BootInfo) -> () {
    unsafe { ptr::write(INIT_RSP as *mut BootInfo, boot_info) }
}

fn jump_to_kernel(boot_info: BootInfo) -> ! {
    save_boot_info(boot_info);

    const ADDR_OF_KERNEL: usize = 0xffff_ffff_8000_0000;

    unsafe {
        asm!("mov rsp, rax
        jmp rdi",in("rax") INIT_RSP,in("rdi") ADDR_OF_KERNEL,options(nomem, preserves_flags, nostack,noreturn));
    }
}

fn exit_bootx64<'a>(mem_map: &'a mut [boot::MemoryDescriptor], boot_info: BootInfo) -> ! {
    disable_interruption();

    memory::init_paging(mem_map);
    jump_to_kernel(boot_info);
}

#[start]
#[no_mangle]
pub fn efi_main(image: Handle, system_table: SystemTable<Boot>) -> ! {
    initialize(&system_table);

    let vram_info = gop::init(&system_table);

    fs::place_kernel(&system_table);
    let mem_map = terminate_boot_services(image, system_table);

    exit_bootx64(mem_map, BootInfo::new(vram_info));
}
