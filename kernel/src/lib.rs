// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
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
extern crate x86_64;

mod gdt;
mod idt;
mod interrupt;
mod panic;
mod queue;

#[macro_use]
mod graphics;

use common::boot;
use graphics::screen;
use graphics::Vram;
use graphics::RGB;
use interrupt::handler;
use interrupt::mouse;
use x86_64::instructions::interrupts;

#[no_mangle]
#[start]
pub extern "win64" fn os_main(boot_info: boot::Info) -> ! {
    Vram::init(&boot_info);
    let mut mouse_device = mouse::Device::new();
    let mut mouse_cursor = screen::MouseCursor::new(RGB::new(0x0000_8484), screen::MOUSE_GRAPHIC);

    initialization(&mut mouse_device, &mut mouse_cursor);

    main_loop(&mut mouse_device, &mut mouse_cursor)
}

fn initialization(
    mouse_device: &mut interrupt::mouse::Device,
    mouse_cursor: &mut graphics::screen::MouseCursor,
) {
    gdt::init();
    idt::init();
    interrupt::init_pic();

    graphics::screen::draw_desktop();

    print_with_pos!(
        graphics::screen::Coord::new(16, 64),
        graphics::RGB::new(0x00FF_FFFF),
        "x_len = {}",
        Vram::x_len()
    );

    interrupt::set_init_pic_bits();
    interrupt::init_keyboard();
    mouse_device.enable();

    mouse_cursor.draw_offset(graphics::screen::Coord::new(300, 300))
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
    if interrupt::KEY_QUEUE.lock().size() > 0 {
        handler::keyboard_data();
    } else if mouse::QUEUE.lock().size() > 0 {
        handler::mouse_data(mouse_device, mouse_cursor);
    } else {
        interrupts::enable_interrupts_and_hlt();
    }
}
