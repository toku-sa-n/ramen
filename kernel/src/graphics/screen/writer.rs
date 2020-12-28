// SPDX-License-Identifier: GPL-3.0-or-later

use super::{font, layer, Vram};
use core::convert::TryFrom;
use rgb::RGB8;
use screen_layer::Layer;
use vek::Vec2;

pub struct Writer {
    id: screen_layer::Id,
    coord: Vec2<i32>,
    color: RGB8,
}

impl Writer {
    pub fn new(coord: Vec2<i32>, color: RGB8) -> Self {
        let l = Layer::new(Vec2::zero(), Vram::resolution().as_());
        let id = layer::add(l);
        Self { id, coord, color }
    }

    fn print_str(&mut self, str: &str) {
        for c in str.chars() {
            if c == '\n' {
                self.break_line();
                continue;
            }

            self.print_char(font::FONTS[c as usize]);
            self.move_cursor_by_one_character();

            if self.cursor_is_outside_screen() {
                self.break_line();
            }
        }
    }

    fn break_line(&mut self) {
        self.coord.x = 0;
        self.coord.y += i32::try_from(font::FONT_HEIGHT).unwrap();
    }

    fn move_cursor_by_one_character(&mut self) {
        self.coord.x += i32::try_from(font::FONT_WIDTH).unwrap();
    }

    fn cursor_is_outside_screen(&self) -> bool {
        self.coord.x + i32::try_from(font::FONT_WIDTH).unwrap() >= Vram::resolution().x
    }

    fn print_char(&self, font: [[bool; font::FONT_WIDTH]; font::FONT_HEIGHT]) {
        for (y, line) in font.iter().enumerate().take(font::FONT_HEIGHT) {
            for (x, cell) in line.iter().enumerate().take(font::FONT_WIDTH) {
                if *cell {
                    let c = self.coord + Vec2::new(x, y).as_();
                    layer::set_pixel(self.id, c.as_(), Some(self.color)).unwrap();
                }
            }
        }
    }
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        self.print_str(s);
        Ok(())
    }
}
