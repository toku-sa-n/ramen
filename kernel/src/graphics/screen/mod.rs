// SPDX-License-Identifier: GPL-3.0-or-later

pub mod desktop;
pub mod layer;

pub mod log;
pub mod writer;

use {
    super::{font, Vram},
    core::{cmp, convert::TryFrom},
    layer::Layer,
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
    id: layer::Id,
}

impl MouseCursor {
    pub fn new() -> Self {
        let layer = Layer::new(
            Vec2::zero(),
            Vec2::new(MOUSE_CURSOR_WIDTH, MOUSE_CURSOR_HEIGHT).as_(),
        );

        let id = layer::CONTROLLER.lock().add_layer(layer);

        layer::CONTROLLER
            .lock()
            .edit_layer(id, |layer: &mut Layer| {
                for y in 0..MOUSE_CURSOR_HEIGHT {
                    for x in 0..MOUSE_CURSOR_WIDTH {
                        layer[y][x] = match MOUSE_GRAPHIC[y][x] {
                            '*' => Some(RGB8::new(0, 0, 0)),
                            '0' => Some(RGB8::new(0xff, 0xff, 0xff)),
                            _ => None,
                        }
                    }
                }
            })
            .expect("Layer of mouse cursor should be added.");

        Self {
            coord: Vec2::new(0, 0),
            id,
        }
    }

    pub fn draw_offset(&mut self, offset: Vec2<i32>) {
        let new_coord = self.coord + offset;
        self.coord = new_coord;
        self.draw(new_coord)
    }

    pub fn draw(&mut self, coord: Vec2<i32>) {
        layer::CONTROLLER
            .lock()
            .slide_layer(self.id, coord)
            .expect("Layer of mouse cursor should be added.");
    }
}
