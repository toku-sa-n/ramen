// SPDX-License-Identifier: GPL-3.0-or-later

use super::{font, layer, Vram};
use core::convert::{TryFrom, TryInto};
use font::{FONTS, FONT_HEIGHT, FONT_WIDTH};
use rgb::RGB8;
use screen_layer::Layer;
use vek::Vec2;

pub struct Writer {
    id: screen_layer::Id,
    coord: Vec2<u32>,
    color: RGB8,
}

impl Writer {
    pub fn new(coord: Vec2<u32>, color: RGB8) -> Self {
        let l = Layer::new(Vec2::zero(), Vram::resolution().as_());
        let id = layer::add(l);

        Self { id, coord, color }
    }

    fn print_str(&mut self, str: &str) {
        for c in str.chars() {
            self.print_char(c);
        }
    }

    fn print_char(&mut self, c: char) {
        if c == '\n' {
            self.break_line_or_scroll();
        } else {
            self.print_normal_char(c);
        }
    }

    fn break_line_or_scroll(&mut self) {
        self.carriage_return();
        if self.cursor_is_bottom_line() {
            self.scroll();
        } else {
            self.break_line();
        }
    }

    fn carriage_return(&mut self) {
        self.coord.x = 0;
    }

    fn cursor_is_bottom_line(&self) -> bool {
        self.coord.y + FONT_HEIGHT >= Vram::resolution().y
    }

    fn scroll(&mut self) {
        layer::edit(self.id, |l| {
            let before_bottom_line: usize =
                (Vram::resolution().y - FONT_HEIGHT).try_into().unwrap();

            for x in 0..usize::try_from(Vram::resolution().x).unwrap() {
                for y in 0..before_bottom_line {
                    l[y][x] = l[y + usize::try_from(FONT_HEIGHT).unwrap()][x];
                }

                for y in before_bottom_line..usize::try_from(Vram::resolution().y).unwrap() {
                    l[y][x] = None;
                }
            }
        })
        .expect("A layer for this writer does not exist.");
    }

    fn break_line(&mut self) {
        self.coord.y += FONT_HEIGHT;
    }

    fn print_normal_char(&mut self, c: char) {
        self.write_char_on_layer(FONTS[c as usize]);
        self.move_cursor_by_one_character();

        if self.cursor_is_outside_screen() {
            self.break_line_or_scroll();
        }
    }

    fn move_cursor_by_one_character(&mut self) {
        self.coord.x += FONT_WIDTH;
    }

    fn cursor_is_outside_screen(&self) -> bool {
        self.coord.x + FONT_WIDTH >= Vram::resolution().x
    }

    fn write_char_on_layer(&self, font: [[bool; FONT_WIDTH as usize]; FONT_HEIGHT as usize]) {
        for (i, line) in font.iter().enumerate() {
            for (j, cell) in line.iter().enumerate() {
                if *cell {
                    let c = self.coord + Vec2::new(j, i).as_();
                    layer::set_pixel(self.id, c.as_(), Some(self.color))
                        .expect("The layer for this writer does not exist");
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
