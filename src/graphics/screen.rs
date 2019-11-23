use super::*;

pub const mouse_graphic: [[char; MOUSE_CURSOR_WIDTH]; MOUSE_CURSOR_HEIGHT] = [
    [
        '*', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
    ],
    [
        '*', '*', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
    ],
    [
        '*', '0', '*', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
    ],
    [
        '*', '0', '0', '*', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
    ],
    [
        '*', '0', '0', '0', '*', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
    ],
    [
        '*', '0', '0', '0', '0', '*', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
    ],
    [
        '*', '0', '0', '0', '0', '0', '*', '.', '.', '.', '.', '.', '.', '.', '.', '.',
    ],
    [
        '*', '0', '0', '0', '0', '0', '0', '*', '.', '.', '.', '.', '.', '.', '.', '.',
    ],
    [
        '*', '0', '0', '0', '0', '0', '0', '0', '*', '.', '.', '.', '.', '.', '.', '.',
    ],
    [
        '*', '0', '0', '0', '0', '0', '*', '*', '*', '*', '.', '.', '.', '.', '.', '.',
    ],
    [
        '*', '0', '0', '*', '0', '0', '*', '.', '.', '.', '.', '.', '.', '.', '.', '.',
    ],
    [
        '*', '0', '*', '.', '*', '0', '0', '*', '.', '.', '.', '.', '.', '.', '.', '.',
    ],
    [
        '*', '*', '.', '.', '*', '0', '0', '*', '.', '.', '.', '.', '.', '.', '.', '.',
    ],
    [
        '*', '.', '.', '.', '.', '*', '0', '0', '*', '.', '.', '.', '.', '.', '.', '.',
    ],
    [
        '.', '.', '.', '.', '.', '*', '0', '*', '.', '.', '.', '.', '.', '.', '.', '.',
    ],
    [
        '.', '.', '.', '.', '.', '.', '*', '.', '.', '.', '.', '.', '.', '.', '.', '.',
    ],
];

#[macro_use]
macro_rules! print_with_pos {
    ($coord:expr,$color:expr,$text:expr,$($args:expr),*) => {
        let mut screen_write =
            crate::graphics::screen::ScreenWrite::new(crate::graphics::Vram::new(), $coord, $color);

        // To make the scope of `use core::fmt::Write;` narrow, enclose sentences by curly braces.
        {
            use core::fmt::Write;
            write!(screen_write, $text, $($args,)*).unwrap();
        }
    };
}

pub struct Coord {
    x: isize,
    y: isize,
}

impl Coord {
    pub fn new(x: isize, y: isize) -> Coord {
        Coord { x: x, y: y }
    }
}

pub struct ScreenWrite {
    vram: Vram,
    coord: Coord,
    color: ColorIndex,
}

impl ScreenWrite {
    pub fn new(vram: Vram, coord: Coord, color: ColorIndex) -> ScreenWrite {
        ScreenWrite { vram, coord, color }
    }
}

impl core::fmt::Write for ScreenWrite {
    fn write_str(&mut self, s: &str) -> core::result::Result<(), core::fmt::Error> {
        print_str(&self.vram, &self.coord, self.color, s);
        self.coord.x += (s.len() * font::FONT_WIDTH) as isize;
        Ok(())
    }
}

pub const MOUSE_CURSOR_WIDTH: usize = 16;
pub const MOUSE_CURSOR_HEIGHT: usize = 16;
pub struct MouseCursor {
    x: isize,
    y: isize,
    dots: [[u8; MOUSE_CURSOR_WIDTH]; MOUSE_CURSOR_HEIGHT],
}

impl MouseCursor {
    pub fn new(
        x: isize,
        y: isize,
        background_color: ColorIndex,
        dots: [[char; MOUSE_CURSOR_WIDTH]; MOUSE_CURSOR_HEIGHT],
    ) -> MouseCursor {
        let mut colored_dots: [[u8; MOUSE_CURSOR_WIDTH]; MOUSE_CURSOR_HEIGHT] =
            [[background_color as u8; MOUSE_CURSOR_WIDTH]; MOUSE_CURSOR_WIDTH];

        for y in 0..MOUSE_CURSOR_HEIGHT {
            for x in 0..MOUSE_CURSOR_WIDTH {
                colored_dots[y][x] = match dots[y][x] {
                    '*' => ColorIndex::Rgb000000 as u8,
                    '0' => ColorIndex::RgbFFFFFF as u8,
                    _ => background_color as u8,
                }
            }
        }

        MouseCursor {
            x,
            y,
            dots: colored_dots,
        }
    }

    pub fn draw(&self) {
        for y in 0..MOUSE_CURSOR_HEIGHT {
            for x in 0..MOUSE_CURSOR_WIDTH {
                let vram: Vram = Vram::new();
                unsafe {
                    *(vram.ptr.offset(
                        (self.y + y as isize) * vram.x_len as isize + (self.x + x as isize),
                    )) = self.dots[y][x];
                }
            }
        }
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

fn print_char(
    vram: &Vram,
    coord: Coord,
    color: ColorIndex,
    font: [[bool; font::FONT_WIDTH]; font::FONT_HEIGHT],
) -> () {
    for i in 0..font::FONT_HEIGHT {
        for j in 0..font::FONT_WIDTH {
            if font[i][j] {
                unsafe {
                    *(&mut *(vram.ptr.offset(
                        ((coord.y as usize + i) * vram.x_len as usize + coord.x as usize + j)
                            as isize,
                    ))) = color as u8;
                }
            }
        }
    }
}

fn print_str(vram: &Vram, coord: &Coord, color: ColorIndex, str: &str) -> () {
    let mut char_x_pos = coord.x;
    for c in str.chars() {
        print_char(
            vram,
            Coord::new(char_x_pos, coord.y),
            color,
            font::FONTS[c as usize],
        );
        char_x_pos += font::FONT_WIDTH as isize;
    }
}
