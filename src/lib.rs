#![no_std]
#![feature(asm)]
#![feature(start)]

mod asm;

#[no_mangle]
#[start]
pub fn os_main() -> isize {
    init_palette();
    for i in 0xa0000..0xb0000 {
        unsafe {
            *(&mut *(i as *mut u8)) = (i & 0x0f) as u8;
        }
    }

    loop {
        asm::hlt()
    }
}

fn init_palette() -> () {
    const RGB_TABLE: [[u8; 3]; 16] = [
        [0x00, 0x00, 0x00],
        [0xff, 0x00, 0x00],
        [0x00, 0xff, 0x00],
        [0xff, 0xff, 0x00],
        [0x00, 0x00, 0xff],
        [0xff, 0x00, 0xff],
        [0x00, 0xff, 0xff],
        [0xff, 0xff, 0xff],
        [0xc6, 0xc6, 0xc6],
        [0x84, 0x00, 0x00],
        [0x00, 0x84, 0x00],
        [0x84, 0x84, 0x00],
        [0x00, 0x00, 0x84],
        [0x84, 0x00, 0x84],
        [0x00, 0x84, 0x84],
        [0x84, 0x84, 0x84],
    ];

    set_palette(0, 15, RGB_TABLE);
}

fn set_palette(start: i32, end: i32, rgb: [[u8; 3]; 16]) -> () {
    let eflags: i32 = asm::load_eflags();
    asm::cli();
    asm::out8(0x03c8, start);
    for i in start..(end + 1) {
        for j in 0..3 {
            asm::out8(0x03c9, (rgb[i as usize][j] >> 2) as i32);
        }
    }
    asm::store_eflags(eflags);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        asm::hlt()
    }
}
