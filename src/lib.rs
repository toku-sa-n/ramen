#![no_std]
#![feature(asm)]
#![feature(start)]
#![feature(naked_functions)]

mod asm;
mod descriptor_table;
mod interrupt;

#[macro_use]
mod graphics;

#[no_mangle]
#[start]
pub fn os_main() -> isize {
    descriptor_table::init();
    interrupt::init_pic();
    asm::sti();
    let vram: graphics::Vram = graphics::Vram::new();
    vram.init_palette();

    graphics::screen::draw_desktop(&vram);

    print_with_pos!(
        graphics::screen::Coord::new(16, 64),
        graphics::ColorIndex::RgbFFFFFF,
        "x_len = {}",
        vram.x_len
    );

    let mouse_cursor: graphics::screen::MouseCursor = graphics::screen::MouseCursor::new(
        300,
        300,
        graphics::ColorIndex::Rgb008484,
        graphics::screen::MOUSE_GRAPHIC,
    );

    mouse_cursor.draw();

    interrupt::enable_pic1_keyboard_mouse();

    loop {
        asm::hlt()
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        asm::hlt()
    }
}
