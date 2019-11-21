#![no_std]
#![feature(asm)]
#![feature(start)]

mod asm;
mod graphics;

#[no_mangle]
#[start]
pub fn os_main() -> isize {
    graphics::init_palette();
    for i in 0xa0000..0xb0000 {
        unsafe {
            *(&mut *(i as *mut u8)) = (i & 0x0f) as u8;
        }
    }

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
