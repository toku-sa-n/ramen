// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![feature(
    abi_x86_interrupt,
    alloc_error_handler,
    asm,
    const_btree_new,
    naked_functions
)]
#![deny(clippy::pedantic, clippy::all)]

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

use common::kernelboot;
use interrupt::{apic, idt, timer};
use log::info;
use process::Privilege;
use terminal::vram;

/// # Panics
///
/// Maybe.
#[no_mangle]
pub extern "win64" fn os_main(mut boot_info: kernelboot::Info) -> ! {
    let boot_info: &mut kernelboot::Info = &mut boot_info;
    vram::init(boot_info);

    terminal::log::init().unwrap();

    info!("Hello Ramen OS!");

    fs::list_names();

    gdt::init();
    idt::init();

    mem::init(boot_info.mem_map_mut());

    let acpi = unsafe { acpi::get(boot_info.rsdp()) };

    apic::io::init(&acpi);

    timer::init(&acpi);

    vram::print_info();

    syscall::init();

    add_processes();

    process::switch();

    do_nothing();

    unreachable!();
}

fn add_processes() {
    process::idle();
    process::binary("build/port_server.bin", Privilege::Kernel);
    process::binary("build/pm.bin", Privilege::User);
    process::binary("build/fs.bin", Privilege::User);
    process::binary("build/xhci.bin", Privilege::User);
    process::binary("build/vm.bin", Privilege::User);
    process::from_function(sysproc::main, "sysproc");
    process::from_function(do_nothing, "do_nothing");

    if cfg!(feature = "qemu_test") {
        process::from_function(tests::main, "tests");
        process::from_function(tests::process::exit_test, "exittest");

        for _ in 0..100 {
            process::binary("build/do_nothing.bin", Privilege::User);
        }
    }
}

fn do_nothing() {
    loop {
        x86_64::instructions::interrupts::enable_and_hlt();
    }
}
