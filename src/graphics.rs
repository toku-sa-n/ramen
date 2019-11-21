use crate::asm;

pub const COLOR_000000: u8 = 0;
pub const COLOR_FF0000: u8 = 1;
pub const COLOR_00FF00: u8 = 2;
pub const COLOR_FFFF00: u8 = 3;
pub const COLOR_0000FF: u8 = 4;
pub const COLOR_FF00FF: u8 = 5;
pub const COLOR_00FFFF: u8 = 6;
pub const COLOR_FFFFFF: u8 = 7;
pub const COLOR_C6C6C6: u8 = 8;
pub const COLOR_840000: u8 = 9;
pub const COLOR_008400: u8 = 10;
pub const COLOR_848400: u8 = 11;
pub const COLOR_000084: u8 = 12;
pub const COLOR_840084: u8 = 13;
pub const COLOR_008484: u8 = 14;
pub const COLOR_848484: u8 = 15;

pub fn init_palette() -> () {
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

pub fn draw_rectangle(
    vram: *mut u8,
    x_len: isize,
    color: u8,
    x0: isize,
    y0: isize,
    x1: isize,
    y1: isize,
) -> () {
    for y in y0..(y1 + 1) {
        for x in x0..(x1 + 1) {
            unsafe {
                *(&mut *(vram.offset(y * x_len + x))) = color as u8;
            }
        }
    }
}
