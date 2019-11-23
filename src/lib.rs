#![no_std]
#![feature(asm)]
#![feature(start)]

mod asm;
mod graphics;

#[no_mangle]
#[start]
pub fn os_main() -> isize {
    let vram: graphics::Vram = graphics::Vram::new();
    vram.init_palette();

    graphics::screen::draw_desktop(&vram);

    graphics::screen::print_str(
        &vram,
        &graphics::screen::Coord::new(8, 8),
        graphics::ColorIndex::RgbFFFFFF,
        "ABC 123",
    );

    let mut screen_write = graphics::screen::ScreenWrite::new(
        graphics::Vram::new(),
        graphics::screen::Coord::new(16, 64),
        graphics::ColorIndex::RgbFFFFFF,
    );

    use core::fmt::Write;
    write!(screen_write, "x_len = {}", vram.x_len).unwrap();

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
