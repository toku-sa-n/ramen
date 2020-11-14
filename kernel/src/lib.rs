// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
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
mod graphics;
mod device;
mod gdt;
mod idt;
mod interrupt;
mod mem;
mod multitask;
mod panic;

use {
    alloc::rc::Rc,
    common::kernelboot,
    core::cell::RefCell,
    device::{
        keyboard, mouse,
        pci::{ahci, xhci},
    },
    graphics::{
        screen::{self, desktop::Desktop, layer},
        Vram,
    },
    mem::allocator::{heap, phys::FrameManager},
    multitask::{
        executor::Executor,
        task::{self, Task},
    },
    x86_64::instructions::interrupts,
};

#[no_mangle]
#[start]
pub extern "win64" fn os_main(mut boot_info: kernelboot::Info) -> ! {
    initialization(&mut boot_info);

    run_tasks();
}

fn initialization(boot_info: &mut kernelboot::Info) {
    Vram::init(&boot_info);

    gdt::init();
    idt::init();
    interrupt::init_pic();

    interrupts::enable();

    heap::init(boot_info.mem_map_mut());

    FrameManager::init(boot_info.mem_map_mut());

    layer::init();

    screen::log::init().unwrap();

    let desktop = Desktop::new();
    desktop.draw();

    info!("Hello Ramen OS!");
    info!("Vram information: {}", Vram::display());

    info!(
        "The number of PCI devices: {}",
        device::pci::iter_devices().count()
    );

    interrupt::set_init_pic_bits();
}

#[cfg(not(feature = "qemu_test"))]
fn run_tasks() -> ! {
    let task_collection = Rc::new(RefCell::new(task::Collection::new()));
    task_collection
        .borrow_mut()
        .add_task_as_woken(Task::new(keyboard::task()));
    task_collection
        .borrow_mut()
        .add_task_as_woken(Task::new(mouse::task()));
    task_collection
        .borrow_mut()
        .add_task_as_woken(Task::new(xhci::task(task_collection.clone())));
    task_collection
        .borrow_mut()
        .add_task_as_woken(Task::new(ahci::task()));

    let mut executor = Executor::new(task_collection);
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
