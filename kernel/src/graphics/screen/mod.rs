// SPDX-License-Identifier: GPL-3.0-or-later

pub mod desktop;
mod layer;

pub mod log;
pub mod writer;

use {
    super::{font, Vram},
    core::{cmp, convert::TryFrom},
    rgb::RGB8,
    vek::Vec2,
};

pub const MOUSE_CURSOR_WIDTH: usize = 16;
pub const MOUSE_CURSOR_HEIGHT: usize = 16;

const MOUSE_GRAPHIC: [[char; MOUSE_CURSOR_WIDTH]; MOUSE_CURSOR_HEIGHT] = [
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
            crate::graphics::screen::writer::Writer::new($coord, $color);

        // To narrow the scope of `use core::fmt::Write;`, enclose sentences by curly braces.
        {
            use core::fmt::Write;
            write!(screen_write, $text, $($args,)*).unwrap();
        }
    };
}

pub struct Screen;

impl Screen {
    // TODO: Specify top left coordinate and length, rather than two coordinates.
    pub fn draw_rectangle(color: RGB8, top_left: Vec2<i32>, bottom_right: Vec2<i32>) {
        for y in top_left.y..=bottom_right.y {
            for x in top_left.x..=bottom_right.x {
                unsafe {
                    Vram::set_color(Vec2::new(x, y), color);
                }
            }
        }
    }
}

pub struct MouseCursor {
    coord: Vec2<i32>,
    image: [[RGB8; MOUSE_CURSOR_WIDTH]; MOUSE_CURSOR_HEIGHT],
}

impl MouseCursor {
    pub fn new() -> Self {
        let mut colored_dots: [[RGB8; MOUSE_CURSOR_WIDTH]; MOUSE_CURSOR_HEIGHT] =
            [[RGB8::default(); MOUSE_CURSOR_WIDTH]; MOUSE_CURSOR_WIDTH];

        for y in 0..MOUSE_CURSOR_HEIGHT {
            for x in 0..MOUSE_CURSOR_WIDTH {
                colored_dots[y][x] = match MOUSE_GRAPHIC[y][x] {
                    '*' => RGB8::new(0, 0, 0),
                    '0' => RGB8::new(0xff, 0xff, 0xff),
                    _ => RGB8::new(0, 0x84, 0x84),
                }
            }
        }

        Self {
            coord: Vec2::new(0, 0),
            image: colored_dots,
        }
    }

    pub fn draw_offset(&mut self, offset: Vec2<i32>) {
        let new_coord = self.coord + offset;
        self.draw(new_coord)
    }

    fn put_coord_on_screen(mut coord: Vec2<i32>) -> Vec2<i32> {
        coord.x = cmp::max(coord.x, 0);
        coord.y = cmp::max(coord.y, 0);

        coord.x = cmp::min(
            coord.x,
            Vram::resolution().x - i32::try_from(MOUSE_CURSOR_WIDTH).unwrap() - 1,
        );
        coord.y = cmp::min(
            coord.y,
            Vram::resolution().y - i32::try_from(MOUSE_CURSOR_HEIGHT).unwrap() - 1,
        );

        coord
    }

    pub fn draw(&mut self, coord: Vec2<i32>) {
        self.remove_previous_cursor();

        let adjusted_coord = Self::put_coord_on_screen(coord);
        for y in 0..MOUSE_CURSOR_HEIGHT {
            for x in 0..MOUSE_CURSOR_WIDTH {
                unsafe {
                    Vram::set_color(adjusted_coord + Vec2::new(x, y).as_(), self.image[y][x]);
                }
            }
        }

        self.coord = adjusted_coord;
    }

    fn remove_previous_cursor(&self) {
        Screen::draw_rectangle(
            RGB8::new(0, 0x84, 0x84),
            Vec2::new(self.coord.x, self.coord.y),
            Vec2::new(
                self.coord.x + i32::try_from(MOUSE_CURSOR_WIDTH).unwrap(),
                self.coord.y + i32::try_from(MOUSE_CURSOR_HEIGHT).unwrap(),
            ),
        );
    }
}
