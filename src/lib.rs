#![no_std]
#![feature(asm)]
#![feature(start)]

mod asm;
mod graphics;

#[no_mangle]
#[start]
pub fn os_main() -> isize {
    graphics::init_palette();

    let p = 0xa0000 as *mut u8;

    graphics::draw_rectangle(p, 320, graphics::COLOR_FF0000, 20, 20, 120, 120);
    graphics::draw_rectangle(p, 320, graphics::COLOR_00FF00, 70, 50, 170, 150);
    graphics::draw_rectangle(p, 320, graphics::COLOR_0000FF, 120, 80, 220, 180);

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
