// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![feature(
    option_expect_none,
    int_bits_const,
    const_btree_new,
    async_closure,
    alloc_error_handler,
    linked_list_remove,
    const_fn,
    asm,
    panic_info_message,
    start,
    naked_functions,
    abi_x86_interrupt
)]
#![deny(clippy::pedantic, clippy::all)]

#[macro_use]
#[allow(unused_imports)]
extern crate common;
extern crate alloc;
#[macro_use]
extern crate log;
extern crate x86_64;
#[macro_use]
extern crate derive_builder;

mod acpi;
mod device;
mod fs;
mod gdt;
mod interrupt;
mod mem;
mod multitask;
mod panic;
mod process;
mod qemu;
mod syscall;
mod tests;
mod tss;

use common::kernelboot;
use device::pci::xhci;
use futures_intrusive::sync::{GenericMutex, GenericMutexGuard};
use interrupt::{apic, idt, timer};
use multitask::{executor::Executor, task::Task};
use process::Privilege;
use spinning_top::RawSpinlock;
use terminal::vram;
pub type Futurelock<T> = GenericMutex<RawSpinlock, T>;
pub type FuturelockGuard<'a, T> = GenericMutexGuard<'a, RawSpinlock, T>;

#[no_mangle]
#[start]
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
    process::add(port_server::main, Privilege::Kernel);
    process::add(run_tasks, Privilege::User);

    if cfg!(feature = "qemu_test") {
        process::add(tests::main, Privilege::User);
        process::add(tests::process::kernel_privilege_test, Privilege::Kernel);
        process::add(tests::process::exit_test, Privilege::User);
        process::add(tests::process::ipc::proc_1, Privilege::User);
        process::add(tests::process::ipc::proc_2, Privilege::User);

        for _ in 0..100 {
            process::add(tests::process::do_nothing, Privilege::User);
        }
    }
}

fn cause_timer_interrupt() -> ! {
    // SAFETY: This interrupt is handled correctly.
    unsafe { asm!("int 0x20", options(noreturn)) }
}

fn run_tasks() {
    multitask::add(Task::new(xhci::task()));

    let mut executor = Executor::new();
    executor.run();
}
