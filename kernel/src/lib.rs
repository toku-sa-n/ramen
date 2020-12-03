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
mod graphics;
mod device;
mod fs;
mod gdt;
mod idt;
mod interrupt;
mod mem;
mod multitask;
mod panic;

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
use mem::allocator::{heap, phys::FrameManager};
use multitask::{
    executor::Executor,
    task::{self, Task},
};
use spinning_top::RawSpinlock;
use x86_64::instructions::interrupts;
pub type Futurelock<T> = GenericMutex<RawSpinlock, T>;

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

    fs::ustar::list_files(INITRD_ADDR);
}

#[cfg(not(feature = "qemu_test"))]
fn run_tasks() -> ! {
    task::COLLECTION
        .lock()
        .add_task_as_woken(Task::new(keyboard::task()));
    task::COLLECTION
        .lock()
        .add_task_as_woken(Task::new(mouse::task()));
    task::COLLECTION
        .lock()
        .add_task_as_woken(Task::new(xhci::task()));
    task::COLLECTION
        .lock()
        .add_task_as_woken(Task::new(ahci::task()));

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
