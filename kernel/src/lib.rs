// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![feature(int_bits_const)]
#![feature(async_closure)]
#![feature(alloc_error_handler)]
#![feature(linked_list_remove)]
#![feature(const_fn)]
#![feature(wake_trait)]
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

#[macro_use]
mod graphics;
mod acpi;
mod device;
mod fs;
mod gdt;
mod interrupt;
mod mem;
mod multitask;
mod panic;
mod process;
mod syscall;
mod tests;
mod tss;

use common::{constant::INITRD_ADDR, kernelboot};
use device::{
    keyboard, mouse,
    pci::{ahci, xhci},
};
use fs::ustar::Ustar;
use futures_intrusive::sync::GenericMutex;
use graphics::{
    screen::{self, desktop::Desktop, layer},
    Vram,
};
use interrupt::{apic, idt, timer};
use mem::allocator::{heap, phys::FrameManager};
use multitask::{executor::Executor, task::Task};
use process::Process;
use spinning_top::RawSpinlock;
pub type Futurelock<T> = GenericMutex<RawSpinlock, T>;

#[no_mangle]
#[start]
pub extern "win64" fn os_main(mut boot_info: kernelboot::Info) -> ! {
    init(&mut boot_info);
    wait_until_timer_interrupt_happens();
}

fn init(boot_info: &mut kernelboot::Info) {
    initialize_in_kernel_mode(boot_info);
    initialize_in_user_mode(boot_info);
}

fn initialize_in_kernel_mode(boot_info: &mut kernelboot::Info) {
    gdt::init();
    idt::init();

    // It is bothering to initialize heap memory in the user mode as this is to map the area, which an initialized
    // frame manager is needed.
    heap::init(boot_info.mem_map_mut());

    // This function unmaps all user memory, which needs the kernel privilege.
    FrameManager::init(boot_info.mem_map_mut());

    let acpi = unsafe { acpi::get(boot_info.rsdp()) };

    apic::io::init(&acpi);

    timer::init(&acpi);

    // This function reads a value from CR3 register, which needs the kernel privilege level.
    process::init();
}

fn initialize_in_user_mode(boot_info: &mut kernelboot::Info) {
    syscall::init();
    gdt::enter_usermode();

    Vram::init(&boot_info);

    layer::init();

    screen::log::init().unwrap();

    let desktop = Desktop::new();
    desktop.draw();

    info!("Hello Ramen OS!");
    info!("Vram information: {}", Vram::display());

    // SAFETY: `INITRD_ADDR` is the valid address to UStar data.
    let ustar = unsafe { Ustar::new(INITRD_ADDR) };
    ustar.list();
    ustar.content("build/bootx64.efi");

    process::add(Process::kernel(process::manager_main));
    process::add(Process::user(run_tasks));

    if cfg!(feature = "qemu_test") {
        process::add(Process::user(tests::main));
        process::add(Process::kernel(tests::process::kernel_privilege_test));
    }
}

fn wait_until_timer_interrupt_happens() -> ! {
    loop {
        syscalls::enable_interrupt_and_halt()
    }
}

fn run_tasks() -> ! {
    multitask::add(Task::new(keyboard::task()));
    multitask::add(Task::new(mouse::task()));
    multitask::add(Task::new(xhci::task()));
    multitask::add(Task::new(ahci::task()));

    let mut executor = Executor::new();
    executor.run();
}
