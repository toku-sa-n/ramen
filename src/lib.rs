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

    graphics::screen::draw_desktop(vram);

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
