#![no_std]
#![feature(asm)]
#![feature(start)]

mod asm;

#[macro_use]
mod graphics;

#[no_mangle]
#[start]
pub fn os_main() -> isize {
    let vram: graphics::Vram = graphics::Vram::new();
    vram.init_palette();

    graphics::screen::draw_desktop(&vram);

    print_with_pos!(
        graphics::screen::Coord::new(8, 8),
        graphics::ColorIndex::RgbFFFFFF,
        "ABC 123",
    );

    print_with_pos!(
        graphics::screen::Coord::new(16, 64),
        graphics::ColorIndex::RgbFFFFFF,
        "x_len = {}",
        vram.x_len
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
