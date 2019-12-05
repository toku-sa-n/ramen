#![no_std]
#![feature(asm)]
#![feature(start)]
#![feature(naked_functions)]

mod asm;
mod descriptor_table;
mod interrupt;
mod queue;

#[macro_use]
mod graphics;

#[no_mangle]
#[start]
pub fn os_main() -> isize {
    let mut mouse_device: interrupt::MouseDevice = interrupt::MouseDevice::new();
    initialization(&mouse_device);

    loop {
        main_loop(&mut mouse_device)
    }
}

fn initialization(mouse_device: &interrupt::MouseDevice) -> () {
    descriptor_table::init();
    interrupt::init_pic();
    asm::sti();
    let vram: graphics::Vram = graphics::Vram::new();
    vram.init_palette();

    graphics::screen::draw_desktop(&vram);

    print_with_pos!(
        graphics::screen::Coord::new(16, 64),
        graphics::screen::ColorIndex::RgbFFFFFF,
        "x_len = {}",
        vram.x_len
    );

    let mouse_cursor: graphics::screen::MouseCursor = graphics::screen::MouseCursor::new(
        300,
        300,
        graphics::screen::ColorIndex::Rgb008484,
        graphics::screen::MOUSE_GRAPHIC,
    );

    mouse_cursor.draw();

    interrupt::set_init_pic_bits();
    interrupt::init_keyboard();
    mouse_device.enable();
}

fn main_loop(mut mouse_device: &mut interrupt::MouseDevice) -> () {
    asm::cli();
    if interrupt::KEY_QUEUE.lock().size() != 0 {
        handle_keyboard_data();
    } else if interrupt::MOUSE_QUEUE.lock().size() != 0 {
        handle_mouse_data(&mut mouse_device);
    } else {
        asm::stihlt();
    }
}

fn handle_keyboard_data() -> () {
    let data: Option<i32> = interrupt::KEY_QUEUE.lock().dequeue();

    asm::sti();

    let screen: graphics::screen::Screen = graphics::screen::Screen::new(graphics::Vram::new());

    screen.draw_rectangle(
        graphics::screen::ColorIndex::Rgb008484,
        graphics::screen::Coord::new(0, 16),
        graphics::screen::Coord::new(15, 31),
    );

    if let Some(data) = data {
        print_with_pos!(
            graphics::screen::Coord::new(0, 16),
            graphics::screen::ColorIndex::RgbFFFFFF,
            "{:X}",
            data
        );
    }
}

fn handle_mouse_data(mouse_device: &mut interrupt::MouseDevice) -> () {
    let data: Option<i32> = interrupt::MOUSE_QUEUE.lock().dequeue();

    asm::sti();

    let screen: graphics::screen::Screen = graphics::screen::Screen::new(graphics::Vram::new());

    screen.draw_rectangle(
        graphics::screen::ColorIndex::Rgb008484,
        graphics::screen::Coord::new(32, 16),
        graphics::screen::Coord::new(47, 31),
    );

    if let Some(data) = data {
        if mouse_device.put_data(data) {
            mouse_device.print_buf_data();
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        asm::hlt()
    }
}
