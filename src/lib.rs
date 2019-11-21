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

    graphics::screen::draw_desktop(vram, x_len, y_len);

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
