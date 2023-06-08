// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![feature(alloc_error_handler, asm_const, naked_functions, abi_x86_interrupt)]
#![deny(clippy::pedantic, clippy::all, unsafe_op_in_unsafe_fn)]

extern crate alloc;

mod acpi;
mod fs;
mod gdt;
mod interrupt;
mod mem;
mod panic;
mod process;
mod qemu;
mod syscall;
mod sysproc;

#[cfg(feature = "qemu_test")]
mod tests;
mod tss;

use {
    aligned_ptr::ptr,
    interrupt::{apic, idt, timer},
    log::info,
    terminal::vram,
    x86_64::instructions::interrupts,
};

/// # Safety
///
/// `boot_info` must point to the correct address.
#[no_mangle]
pub unsafe extern "sysv64" fn os_main(boot_info: *mut boot_info::Info) -> ! {
    init(unsafe { ptr::get(boot_info) });
    idle();
}

fn init(mut boot_info: boot_info::Info) {
    vram::init(&boot_info);

    terminal::log::init().unwrap();

    info!("Hello Ramen OS!");

    fs::list_names();

    // SAFETY: At this point, `TSS` is never touched.
    unsafe { gdt::init() };

    idt::init();

    mem::init(boot_info.mem_map_mut());

    let acpi = unsafe { acpi::get(boot_info.rsdp()) };

    apic::io::init(&acpi);

    timer::init(&acpi);

    vram::print_info();

    syscall::init();

    process::init();
}

fn idle() -> ! {
    loop {
        interrupts::enable_and_hlt();
    }
}
