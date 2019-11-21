use super::*;

pub struct Coord {
    x: isize,
    y: isize,
}

impl Coord {
    pub fn new(x: isize, y: isize) -> Coord {
        Coord { x: x, y: y }
    }
}

#[rustfmt::skip]
pub fn draw_desktop(vram: &Vram) -> () {
    let x_len:isize  = vram.x_len as isize;
    let y_len:isize  = vram.y_len as isize;
    let vram:*mut u8 = vram.ptr as *mut u8;

    let draw_desktop_part = |color, x0, y0, x1, y1| {
        draw_rectangle(vram, x_len, color, Coord::new(x0, y0), Coord::new(x1, y1));
    };

    draw_desktop_part(ColorIndex::Rgb008484,          0,          0, x_len -  1, y_len - 29);
    draw_desktop_part(ColorIndex::RgbC6C6C6,          0, y_len - 28, x_len -  1, y_len - 28);
    draw_desktop_part(ColorIndex::RgbFFFFFF,          0, y_len - 27, x_len -  1, y_len - 27);
    draw_desktop_part(ColorIndex::RgbC6C6C6,          0, y_len - 26, x_len -  1, y_len -  1);

    draw_desktop_part(ColorIndex::RgbFFFFFF,          3, y_len - 24,         59, y_len - 24);
    draw_desktop_part(ColorIndex::RgbFFFFFF,          2, y_len - 24,          2, y_len -  4);
    draw_desktop_part(ColorIndex::Rgb848484,          3, y_len -  4,         59, y_len -  4);
    draw_desktop_part(ColorIndex::Rgb848484,         59, y_len - 23,         59, y_len -  5);
    draw_desktop_part(ColorIndex::Rgb000000,          2, y_len -  3,         59, y_len -  3);
    draw_desktop_part(ColorIndex::Rgb000000,         60, y_len - 24,         60, y_len -  3);

    draw_desktop_part(ColorIndex::Rgb848484, x_len - 47, y_len - 24, x_len -  4, y_len - 24);
    draw_desktop_part(ColorIndex::Rgb848484, x_len - 47, y_len - 23, x_len - 47, y_len -  4);
    draw_desktop_part(ColorIndex::RgbFFFFFF, x_len - 47, y_len -  3, x_len -  4, y_len -  3);
    draw_desktop_part(ColorIndex::RgbFFFFFF, x_len -  3, y_len - 24, x_len -  3, y_len -  3);
}

fn draw_rectangle(
    vram: *mut u8,
    x_len: isize,
    color: ColorIndex,
    top_left: Coord,
    bottom_right: Coord,
) -> () {
    for y in top_left.y..=bottom_right.y {
        for x in top_left.x..=bottom_right.x {
            unsafe {
                *(&mut *(vram.offset(y * x_len + x))) = color as u8;
            }
        }
    }
}

pub fn put_font(vram: Vram, x: usize, y: usize, color: ColorIndex, font: [u8; 16]) -> () {
    for i in 0..16 {
        for j in 0..8 {
            if font[i] & (1 << j) != 0 {
                unsafe {
                    *(&mut *(vram
                        .ptr
                        .offset(((y + i) * vram.x_len as usize + x + j) as isize))) = color as u8;
                }
            }
        }
    }
}
