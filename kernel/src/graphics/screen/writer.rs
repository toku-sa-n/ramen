// SPDX-License-Identifier: GPL-3.0-or-later

use super::{font, Coord, Vram, RGB};
use core::convert::TryFrom;

pub struct Writer {
    coord: Coord<isize>,
    color: RGB,
}

impl Writer {
    pub const fn new(coord: Coord<isize>, color: RGB) -> Self {
        Self { coord, color }
    }

    fn print_str(&mut self, str: &str) {
        let mut char_x_pos = self.coord.x;
        let mut char_y_pos = self.coord.y;
        for c in str.chars() {
            if c == '\n' {
                char_x_pos = 0;
                char_y_pos += isize::try_from(font::FONT_HEIGHT).unwrap();
                continue;
            }

            print_char(
                &Coord::new(char_x_pos, char_y_pos),
                self.color,
                font::FONTS[c as usize],
            );
            char_x_pos += isize::try_from(font::FONT_WIDTH).unwrap();

            if char_x_pos + isize::try_from(font::FONT_WIDTH).unwrap()
                >= isize::try_from(Vram::resolution().x).unwrap()
            {
                char_x_pos = 0;
                char_y_pos += isize::try_from(font::FONT_HEIGHT).unwrap();
            }
        }
    }
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        self.print_str(s);
        self.coord.x += isize::try_from(s.len() * font::FONT_WIDTH).unwrap();
        self.coord.y += self.coord.x / isize::try_from(Vram::resolution().x).unwrap();
        self.coord.x %= isize::try_from(Vram::resolution().x).unwrap();
        Ok(())
    }
}

fn print_char(
    coord: &Coord<isize>,
    color: RGB,
    font: [[bool; font::FONT_WIDTH]; font::FONT_HEIGHT],
) {
    for (i, line) in font.iter().enumerate().take(font::FONT_HEIGHT) {
        for (j, cell) in line.iter().enumerate().take(font::FONT_WIDTH) {
            if *cell {
                unsafe {
                    Vram::set_color(
                        &(coord.clone()
                            + Coord::new(isize::try_from(j).unwrap(), isize::try_from(i).unwrap())),
                        color,
                    );
                }
            }
        }
    }
}
