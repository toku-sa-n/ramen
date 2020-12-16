// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![feature(async_closure)]
#![feature(alloc_error_handler)]
#![feature(min_const_generics)]
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
mod syscall;
mod tss;

use common::{constant::INITRD_ADDR, kernelboot};
use device::{
    keyboard, mouse,
    pci::{ahci, xhci},
};
use futures_intrusive::sync::GenericMutex;
use graphics::{
    screen::{self, desktop::Desktop, layer},
    Vram,
};
use interrupt::{apic, idt, timer};
use mem::allocator::{heap, phys::FrameManager};
use multitask::{executor::Executor, task::Task};
use spinning_top::RawSpinlock;
pub type Futurelock<T> = GenericMutex<RawSpinlock, T>;

#[no_mangle]
#[start]
pub extern "win64" fn os_main(mut boot_info: kernelboot::Info) -> ! {
    init(&mut boot_info);

    run_tasks();
}

fn init(boot_info: &mut kernelboot::Info) {
    initialize_in_kernel_mode(boot_info);
    gdt::enter_usermode();
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
}

fn initialize_in_user_mode(boot_info: &mut kernelboot::Info) {
    Vram::init(&boot_info);

    layer::init();

    screen::log::init().unwrap();

    let desktop = Desktop::new();
    desktop.draw();

    info!("Hello Ramen OS!");
    info!("Vram information: {}", Vram::display());

    let acpi = unsafe { acpi::get(boot_info.rsdp()) };

    apic::io::init(&acpi);

    timer::init(&acpi);

    syscall::init();

    fs::ustar::list_files(INITRD_ADDR);
}

#[cfg(not(feature = "qemu_test"))]
fn run_tasks() -> ! {
    multitask::add(Task::new(keyboard::task()));
    multitask::add(Task::new(mouse::task()));
    multitask::add(Task::new(xhci::task()));
    multitask::add(Task::new(ahci::task()));

    let mut executor = Executor::new();
    executor.run();
}

#[cfg(feature = "qemu_test")]
fn run_tasks() -> ! {
    use qemu_exit::QEMUExit;
    // Currently there is no way to test multitasking. If this OS suppports timer, the situation
    // may change.
    //
    // If you change the value `0xf4` and `33`, don't forget to change the correspond values in
    // `Makefile`!
    qemu_exit::X86::new(0xf4, 33).exit_success();
}
