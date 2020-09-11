// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![feature(alloc_error_handler)]
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

mod allocator;
mod gdt;
mod idt;
mod interrupt;
mod multitask;
mod panic;

#[macro_use]
mod graphics;

use allocator::{FrameManager, ALLOCATOR};
use common::{
    constant::{BYTES_KERNEL_HEAP, KERNEL_HEAP_ADDR},
    kernelboot,
};
use core::convert::TryFrom;
use graphics::{screen, screen::MouseCursor, Vram};
use interrupt::{handler, mouse};
use rgb::RGB8;
use vek::Vec2;
use x86_64::instructions::interrupts;

#[no_mangle]
#[start]
pub extern "win64" fn os_main(boot_info: kernelboot::Info) -> ! {
    let (mut mouse_device, mut cursor) = initialization(&boot_info);

    main_loop(&mut mouse_device, &mut cursor)
}

fn initialization(boot_info: &kernelboot::Info) -> (mouse::Device, MouseCursor) {
    Vram::init(&boot_info);

    gdt::init();
    idt::init();
    interrupt::init_pic();

    FrameManager::init(boot_info.mem_map());

    unsafe {
        ALLOCATOR.lock().init(
            usize::try_from(KERNEL_HEAP_ADDR.as_u64()).unwrap(),
            BYTES_KERNEL_HEAP.as_usize(),
        )
    }

    screen::log::init().unwrap();

    let mouse_device = mouse::Device::new();
    let mut mouse_cursor = MouseCursor::new(RGB8::new(0, 0x84, 0x84), screen::MOUSE_GRAPHIC);

    graphics::screen::draw_desktop();

    info!("Hello Ramen OS!");
    info!("Vram information: {}", Vram::display());

    let mut executor = multitask::executor::Executor::new();
    executor.spawn(multitask::task::Task::new(
        multitask::executor::sample_task(),
    ));
    executor.run();

    interrupt::set_init_pic_bits();
    interrupt::init_keyboard();
    mouse::Device::enable();

    mouse_cursor.draw_offset(Vec2::new(300, 300));

    (mouse_device, mouse_cursor)
}

#[cfg(not(feature = "qemu_test"))]
fn main_loop(mouse_device: &mut mouse::Device, mouse_cursor: &mut screen::MouseCursor) -> ! {
    loop {
        loop_main(mouse_device, mouse_cursor)
    }
}

#[cfg(feature = "qemu_test")]
fn main_loop(mouse_device: &mut mouse::Device, mouse_cursor: &mut screen::MouseCursor) -> ! {
    // Because of `hlt` instruction, running `loop_main` many times is impossible.
    loop_main(mouse_device, mouse_cursor);

    // If you change the value `0xf4` and `0x10`, don't forget to change the correspond values in
    // `Makefile`!
    qemu_exit::x86::exit::<u32, 0xf4>(0x10);
}

fn loop_main(mouse_device: &mut mouse::Device, mouse_cursor: &mut screen::MouseCursor) {
    interrupts::disable();
    if interrupt::KEY_QUEUE.lock().len() > 0 {
        handler::keyboard_data();
    } else if mouse::QUEUE.lock().len() > 0 {
        handler::mouse_data(mouse_device, mouse_cursor);
    } else {
        interrupts::enable_interrupts_and_hlt();
    }
}
