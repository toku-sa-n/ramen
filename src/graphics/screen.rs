use super::*;

#[derive(Clone, Copy)]
pub enum ColorIndex {
    Rgb000000 = 0,
    _RgbFF0000 = 1,
    _Rgb00FF00 = 2,
    _RgbFFFF00 = 3,
    _Rgb0000FF = 4,
    _RgbFF00FF = 5,
    _Rgb00FFFF = 6,
    RgbFFFFFF = 7,
    RgbC6C6C6 = 8,
    _Rgb840000 = 9,
    _Rgb008400 = 10,
    _Rgb848400 = 11,
    _Rgb000084 = 12,
    _Rgb840084 = 13,
    Rgb008484 = 14,
    Rgb848484 = 15,
}

pub const MOUSE_GRAPHIC: [[char; MOUSE_CURSOR_WIDTH]; MOUSE_CURSOR_HEIGHT] = [
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

#[macro_export]
macro_rules! print_with_pos {
    ($coord:expr,$color:expr,$text:expr,$($args:expr),*) => {
        let mut screen_write =
            crate::graphics::screen::ScreenWrite::new(crate::graphics::Vram::new(), $coord, $color);

        // To narrow the scope of `use core::fmt::Write;`, enclose sentences by curly braces.
        {
            use core::fmt::Write;
            write!(screen_write, $text, $($args,)*).unwrap();
        }
    };
}

pub struct Screen {
    vram: Vram,
}

impl Screen {
    pub fn new(vram: Vram) -> Self {
        Self { vram }
    }

    // TODO: Specify top left coordinate and length, rather than two coordinates.
    pub fn draw_rectangle(&self, color: ColorIndex, top_left: Coord, bottom_right: Coord) -> () {
        for y in top_left.y..=bottom_right.y {
            for x in top_left.x..=bottom_right.x {
                unsafe {
                    *(&mut *(self.vram.ptr.offset(y * self.vram.x_len as isize + x))) = color as u8;
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct Coord {
    x: isize,
    y: isize,
}

type TwoDimensionalVec = Coord;

impl Coord {
    pub fn new(x: isize, y: isize) -> Self {
        Self { x: x, y: y }
    }

    pub fn offset(self, offset: Self) -> Self {
        Self {
            x: self.x + offset.x,
            y: self.y + offset.y,
        }
    }

    pub fn put_in(self, coord_1: Self, coord_2: Self) -> Self {
        let mut new_coord = self;

        if new_coord.x < coord_1.x {
            new_coord.x = coord_1.x;
        }
        if new_coord.x > coord_2.x {
            new_coord.x = coord_2.x;
        }

        if new_coord.y < coord_1.y {
            new_coord.y = coord_1.y;
        }
        if new_coord.y > coord_2.y {
            new_coord.y = coord_2.y;
        }

        new_coord
    }
}

pub struct ScreenWrite {
    vram: Vram,
    coord: Coord,
    color: ColorIndex,
}

impl ScreenWrite {
    pub fn new(vram: Vram, coord: Coord, color: ColorIndex) -> Self {
        Self { vram, coord, color }
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
    coord: Coord,

    vram: Vram,
    image: [[u8; MOUSE_CURSOR_WIDTH]; MOUSE_CURSOR_HEIGHT],
}

impl MouseCursor {
    pub fn new(
        background_color: ColorIndex,
        image: [[char; MOUSE_CURSOR_WIDTH]; MOUSE_CURSOR_HEIGHT],
        vram: Vram,
    ) -> Self {
        let mut colored_dots: [[u8; MOUSE_CURSOR_WIDTH]; MOUSE_CURSOR_HEIGHT] =
            [[background_color as u8; MOUSE_CURSOR_WIDTH]; MOUSE_CURSOR_WIDTH];

        for y in 0..MOUSE_CURSOR_HEIGHT {
            for x in 0..MOUSE_CURSOR_WIDTH {
                colored_dots[y][x] = match image[y][x] {
                    '*' => ColorIndex::Rgb000000 as u8,
                    '0' => ColorIndex::RgbFFFFFF as u8,
                    _ => background_color as u8,
                }
            }
        }

        Self {
            coord: Coord::new(0, 0),
            image: colored_dots,
            vram,
        }
    }

    pub fn draw_offset(self, offset: TwoDimensionalVec) -> Self {
        let new_coord = self.coord.clone().offset(offset);
        self.draw(new_coord)
    }

    pub fn draw(self, coord: Coord) -> Self {
        self.remove_previous_cursor();

        let adjusted_coord = coord.put_in(
            Coord::new(0, 0),
            Coord::new(
                (self.vram.x_len - MOUSE_CURSOR_WIDTH as i16) as isize,
                (self.vram.y_len - MOUSE_CURSOR_HEIGHT as i16) as isize,
            ),
        );

        for y in 0..MOUSE_CURSOR_HEIGHT {
            for x in 0..MOUSE_CURSOR_WIDTH {
                unsafe {
                    *(self.vram.ptr.offset(
                        (adjusted_coord.y + y as isize) * self.vram.x_len as isize
                            + (adjusted_coord.x + x as isize),
                    )) = self.image[y][x];
                }
            }
        }

        Self {
            coord: adjusted_coord,
            ..self
        }
    }

    fn remove_previous_cursor(&self) -> () {
        let screen: Screen = Screen::new(self.vram.clone());

        screen.draw_rectangle(
            ColorIndex::Rgb008484,
            Coord::new(self.coord.x, self.coord.y),
            Coord::new(
                self.coord.x + MOUSE_CURSOR_WIDTH as isize,
                self.coord.y + MOUSE_CURSOR_HEIGHT as isize,
            ),
        );
    }
}

#[rustfmt::skip]
pub fn draw_desktop(vram: &Vram) -> () {
    let x_len:isize  = vram.x_len as isize;
    let y_len:isize  = vram.y_len as isize;

    let draw_desktop_part = |color, x0, y0, x1, y1| {
        let screen:screen::Screen =screen::Screen::new(Vram::new());
        screen.draw_rectangle( color, Coord::new(x0, y0), Coord::new(x1, y1));
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
