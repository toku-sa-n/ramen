#![no_std]
#![feature(asm)]
#![feature(start)]
#![feature(naked_functions)]

#[macro_use]
#[allow(unused_imports)]
extern crate debug;

extern crate common_items;

mod asm;
mod descriptor_table;
mod gdt;
mod interrupt;
mod memory;
mod queue;

#[macro_use]
mod graphics;

use core::{mem, ptr};

#[no_mangle]
#[start]
pub fn os_main() {
    let boot_info = common_items::BootInfo::get();
    let vram = graphics::Vram::new_from_boot_info(&boot_info);

    let mut mouse_device: interrupt::mouse::Device = interrupt::mouse::Device::new(&vram);
    let mut mouse_cursor: graphics::screen::MouseCursor = graphics::screen::MouseCursor::new(
        graphics::RGB::new(0x008484),
        graphics::screen::MOUSE_GRAPHIC,
        &vram,
    );

    initialization(&mut mouse_device, &mut mouse_cursor, &vram);

    main_loop(&mut mouse_device, &mut mouse_cursor, &vram)
}

fn initialization(
    mouse_device: &mut interrupt::mouse::Device,
    mouse_cursor: &mut graphics::screen::MouseCursor,
    vram: &graphics::Vram,
) -> () {
    gdt::init();
    descriptor_table::init();
    interrupt::init_pic();
    // Temporarily disable interruption to see whether desktop is drawn successfully or not.
    // asm::sti();

    graphics::screen::draw_desktop(&vram);

    print_with_pos!(
        vram,
        graphics::screen::Coord::new(16, 64),
        graphics::RGB::new(0xFFFFFF),
        "x_len = {}",
        vram.x_len
    );

    interrupt::set_init_pic_bits();
    interrupt::init_keyboard();
    mouse_device.enable();

    mouse_cursor.draw_offset(graphics::screen::Coord::new(300, 300))
}

fn main_loop(
    mouse_device: &mut interrupt::mouse::Device,
    mouse_cursor: &mut graphics::screen::MouseCursor,
    vram: &graphics::Vram,
) -> () {
    loop {
        asm::cli();
        if interrupt::KEY_QUEUE.lock().size() != 0 {
            handle_keyboard_data(vram);
        } else if interrupt::mouse::QUEUE.lock().size() != 0 {
            handle_mouse_data(mouse_device, mouse_cursor, vram);
        } else {
            asm::stihlt();
        }
    }
}

fn handle_keyboard_data(vram: &graphics::Vram) -> () {
    let data: Option<u32> = interrupt::KEY_QUEUE.lock().dequeue();

    asm::sti();

    let mut screen: graphics::screen::Screen = graphics::screen::Screen::new(vram);

    screen.draw_rectangle(
        graphics::RGB::new(0x008484),
        graphics::screen::Coord::new(0, 16),
        graphics::screen::Coord::new(15, 31),
    );

    if let Some(data) = data {
        print_with_pos!(
            vram,
            graphics::screen::Coord::new(0, 16),
            graphics::RGB::new(0xFFFFFF),
            "{:X}",
            data
        );
    }
}

fn handle_mouse_data(
    mouse_device: &mut interrupt::mouse::Device,
    mouse_cursor: &mut graphics::screen::MouseCursor,
    vram: &graphics::Vram,
) -> () {
    let data: Option<u32> = interrupt::mouse::QUEUE.lock().dequeue();

    asm::sti();

    let mut screen: graphics::screen::Screen = graphics::screen::Screen::new(vram);

    screen.draw_rectangle(
        graphics::RGB::new(0x008484),
        graphics::screen::Coord::new(32, 16),
        graphics::screen::Coord::new(47, 31),
    );

    if data == None {
        return;
    }

    if !mouse_device.put_data(data.unwrap()) {
        return;
    }

    mouse_device.print_buf_data();
    mouse_cursor.draw_offset(mouse_device.get_speed());
    mouse_cursor.print_coord(graphics::screen::Coord::new(16, 32));
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        asm::hlt()
    }
}
