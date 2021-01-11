// SPDX-License-Identifier: GPL-3.0-or-later

use super::vram;
use crate::graphics::font;
use rgb::RGB8;
use vek::Vec2;

pub struct Writer {
    coord: Vec2<u32>,
    color: RGB8,
}

impl Writer {
    pub fn new(coord: Vec2<u32>, color: RGB8) -> Self {
        Self { coord, color }
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
        self.coord.y += font::FONT_HEIGHT;
    }

    fn move_cursor_by_one_character(&mut self) {
        self.coord.x += font::FONT_WIDTH;
    }

    fn cursor_is_outside_screen(&self) -> bool {
        self.coord.x + font::FONT_WIDTH >= vram::resolution().x
    }

    fn print_char(&self, font: [[bool; font::FONT_WIDTH as usize]; font::FONT_HEIGHT as usize]) {
        for (y, line) in font.iter().enumerate() {
            for (x, cell) in line.iter().enumerate() {
                if *cell {
                    let c = self.coord + Vec2::new(x, y).as_();
                    vram::set_pixel(c, self.color);
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
