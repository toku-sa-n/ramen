#![no_std]
#![feature(asm)]
#![feature(start)]

mod asm;
mod graphics;

#[no_mangle]
#[start]
pub fn os_main() -> isize {
    graphics::init_palette();

    let vram = 0xa0000 as *mut u8;
    let x_len: isize = 320;
    let y_len: isize = 200;

    let draw_desktop_part = |color, top_left, bottom_right| {
        graphics::draw_rectangle(vram, x_len, color, top_left, bottom_right);
    };

    draw_desktop_part(
        graphics::ColorIndex::Rgb008484,
        graphics::Coord::new(0, 0),
        graphics::Coord::new(x_len - 1, y_len - 29),
    );
    draw_desktop_part(
        graphics::ColorIndex::RgbC6C6C6,
        graphics::Coord::new(0, y_len - 28),
        graphics::Coord::new(x_len - 1, y_len - 28),
    );
    draw_desktop_part(
        graphics::ColorIndex::RgbFFFFFF,
        graphics::Coord::new(0, y_len - 27),
        graphics::Coord::new(x_len - 1, y_len - 27),
    );
    draw_desktop_part(
        graphics::ColorIndex::RgbC6C6C6,
        graphics::Coord::new(0, y_len - 26),
        graphics::Coord::new(x_len - 1, y_len - 1),
    );

    draw_desktop_part(
        graphics::ColorIndex::RgbFFFFFF,
        graphics::Coord::new(3, y_len - 24),
        graphics::Coord::new(59, y_len - 24),
    );
    draw_desktop_part(
        graphics::ColorIndex::RgbFFFFFF,
        graphics::Coord::new(2, y_len - 24),
        graphics::Coord::new(2, y_len - 4),
    );
    draw_desktop_part(
        graphics::ColorIndex::Rgb848484,
        graphics::Coord::new(3, y_len - 4),
        graphics::Coord::new(59, y_len - 4),
    );
    draw_desktop_part(
        graphics::ColorIndex::Rgb848484,
        graphics::Coord::new(59, y_len - 23),
        graphics::Coord::new(59, y_len - 5),
    );
    draw_desktop_part(
        graphics::ColorIndex::Rgb000000,
        graphics::Coord::new(2, y_len - 3),
        graphics::Coord::new(59, y_len - 3),
    );
    draw_desktop_part(
        graphics::ColorIndex::Rgb000000,
        graphics::Coord::new(60, y_len - 24),
        graphics::Coord::new(60, y_len - 3),
    );

    draw_desktop_part(
        graphics::ColorIndex::Rgb848484,
        graphics::Coord::new(x_len - 47, y_len - 24),
        graphics::Coord::new(x_len - 4, y_len - 24),
    );
    draw_desktop_part(
        graphics::ColorIndex::Rgb848484,
        graphics::Coord::new(x_len - 47, y_len - 23),
        graphics::Coord::new(x_len - 47, y_len - 4),
    );
    draw_desktop_part(
        graphics::ColorIndex::RgbFFFFFF,
        graphics::Coord::new(x_len - 47, y_len - 3),
        graphics::Coord::new(x_len - 4, y_len - 3),
    );
    draw_desktop_part(
        graphics::ColorIndex::RgbFFFFFF,
        graphics::Coord::new(x_len - 3, y_len - 24),
        graphics::Coord::new(x_len - 3, y_len - 3),
    );

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
