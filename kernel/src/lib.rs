// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![feature(alloc_error_handler)]
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

#[no_mangle]
pub extern "win64" fn os_main(mut boot_info: kernelboot::Info) -> ! {
    init(&mut boot_info);
    cause_timer_interrupt();
}

fn init(boot_info: &mut kernelboot::Info) {
    vram::init(&boot_info);

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

    add_processes();
}

fn add_processes() {
    process::binary("build/port_server.bin", Privilege::Kernel);
    process::binary("build/pm.bin", Privilege::User);
    process::binary("build/fs.bin", Privilege::User);
    process::binary("build/xhci.bin", Privilege::User);
    process::add(sysproc::main, Privilege::Kernel, "sysproc");

    if cfg!(feature = "qemu_test") {
        process::add(tests::main, Privilege::User, "tests");
        process::add(tests::process::exit_test, Privilege::User, "exittest");

        for _ in 0..100 {
            process::binary("build/do_nothing.bin", Privilege::User);
        }
    }
}

fn cause_timer_interrupt() -> ! {
    extern "C" {
        fn cause_timer_interrupt_asm() -> !;
    }

    unsafe { cause_timer_interrupt_asm() }
}
