// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![feature(option_expect_none)]
#![feature(int_bits_const)]
#![feature(const_btree_new)]
#![feature(async_closure)]
#![feature(alloc_error_handler)]
#![feature(linked_list_remove)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(panic_info_message)]
#![feature(start)]
#![feature(naked_functions)]
#![feature(abi_x86_interrupt)]
#![deny(clippy::pedantic)]
#![deny(clippy::all)]

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
mod gdt;
mod interrupt;
mod multitask;
mod panic;
mod process;
mod qemu;
mod tests;
mod tss;

use common::kernelboot;
use device::pci::xhci;
use futures_intrusive::sync::{GenericMutex, GenericMutexGuard};
use interrupt::{apic, idt, timer};
use memory::allocator::{heap, phys::FrameManager};
use multitask::{executor::Executor, task::Task};
use spinning_top::RawSpinlock;
use terminal::vram;
use x86_64::instructions::interrupts;

pub type Futurelock<T> = GenericMutex<RawSpinlock, T>;
pub type FuturelockGuard<'a, T> = GenericMutexGuard<'a, RawSpinlock, T>;

#[no_mangle]
#[start]
pub extern "win64" fn os_main(mut boot_info: kernelboot::Info) -> ! {
    init(&mut boot_info);
    wait_until_timer_interrupt_happens();
}

fn init(boot_info: &mut kernelboot::Info) {
    gdt::init();
    idt::init();

    // It is bothering to initialize heap memory in the user mode as this is to map the area, which an initialized
    // frame manager is needed.
    heap::init();

    // This function unmaps all user memory, which needs the kernel privilege.
    FrameManager::init(boot_info.mem_map_mut());

    let acpi = unsafe { acpi::get(boot_info.rsdp()) };

    apic::io::init(&acpi);

    timer::init(&acpi);

    vram::init(&boot_info);

    terminal::log::init().unwrap();

    info!("Hello Ramen OS!");

    vram::print_info();

    process::manager::init();
    add_processes();
}

fn add_processes() {
    process::manager::add(run_tasks);

    if cfg!(feature = "qemu_test") {
        process::manager::add(tests::main);

        // TODO: Add the test for lots of processes.
        //
        // for _ in 0..100 {
        //     process::manager::add(tests::process::do_nothing);
        // }
    }
}

fn wait_until_timer_interrupt_happens() -> ! {
    loop {
        interrupts::enable_and_hlt();
    }
}

fn run_tasks() {
    multitask::add(Task::new(xhci::task()));

    let mut executor = Executor::new();
    executor.run();
}
