// SPDX-License-Identifier: GPL-3.0-or-later

use super::{font, vram};
use rgb::RGB8;
use vek::Vec2;

pub struct Writer {
    coord: Vec2<u32>,
    color: RGB8,
}
impl Writer {
    pub const fn new(color: RGB8) -> Self {
        Self {
            coord: Vec2 { x: 0, y: 0 },
            color,
        }
    }

    fn print_str(&mut self, str: &str) {
        for c in str.chars() {
            self.print_char(c);
        }
    }

    fn print_char(&mut self, c: char) {
        if c == '\n' {
            self.break_line();
            return;
        }

        self.write_char_on_screen(font::FONTS[c as usize]);
        self.move_cursor_by_one_character();

        if self.cursor_is_outside_screen() {
            self.break_line();
        }
    }

    fn break_line(&mut self) {
        self.carriage_return();
        self.newline();
    }

    fn carriage_return(&mut self) {
        self.coord.x = 0;
    }

    fn newline(&mut self) {
        if self.cursor_is_at_the_bottom() {
            vram::scroll_up();
        } else {
            self.move_cursor_to_next_line();
        }
    }

    fn cursor_is_at_the_bottom(&self) -> bool {
        self.current_line() == Self::num_lines() - 1
    }

    fn current_line(&self) -> u32 {
        self.coord.y / font::HEIGHT
    }

    fn num_lines() -> u32 {
        vram::resolution().y / font::HEIGHT
    }

    fn move_cursor_to_next_line(&mut self) {
        self.coord.y += font::HEIGHT;
    }

    fn move_cursor_by_one_character(&mut self) {
        self.coord.x += font::WIDTH;
    }

    fn cursor_is_outside_screen(&self) -> bool {
        self.coord.x + font::WIDTH >= vram::resolution().x
    }

    fn write_char_on_screen(&self, font: [[bool; font::WIDTH as usize]; font::HEIGHT as usize]) {
        for (y, line) in font.iter().enumerate() {
            for (x, cell) in line.iter().enumerate() {
                if *cell {
                    let c = self.coord + Vec2::new(x, y).as_();
                    vram::set_color(c.as_(), self.color);
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
