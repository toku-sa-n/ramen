// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![feature(
    alloc_error_handler,
    asm,
    const_btree_new,
    asm_const,
    asm_sym,
    naked_functions
)]
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
mod tests;
mod tss;

use {
    aligned_ptr::ptr,
    interrupt::{apic, idt, timer},
    log::info,
    process::Privilege,
    terminal::vram,
    x86_64::software_interrupt,
};

/// # Safety
///
/// `boot_info` must point to the correct address.
#[no_mangle]
pub unsafe extern "sysv64" fn os_main(boot_info: *mut boot_info::Info) -> ! {
    init(unsafe { ptr::get(boot_info) });
    cause_timer_interrupt();
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

    add_processes();
}

fn add_processes() {
    process::binary("port_server.bin", Privilege::Kernel);
    process::binary("pm.bin", Privilege::User);
    process::binary("fs.bin", Privilege::User);
    process::binary("xhci.bin", Privilege::User);
    process::binary("vm.bin", Privilege::User);
    process::from_function(sysproc::main, "sysproc");
    process::from_function(do_nothing, "do_nothing");

    if cfg!(feature = "qemu_test") {
        process::from_function(tests::main, "tests");
        process::from_function(tests::process::exit_test, "exittest");
    }
}

fn cause_timer_interrupt() -> ! {
    unsafe {
        software_interrupt!(0x20);
    }

    unreachable!();
}

fn do_nothing() {
    loop {
        x86_64::instructions::hlt();
    }
}
