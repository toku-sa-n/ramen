// SPDX-License-Identifier: GPL-3.0-or-later

use crate::graphics::screen::Screen;
use crate::graphics::Vram;
use crate::print_with_pos;
use core::convert::TryFrom;
use rgb::RGB8;
use vek::Vec2;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    Screen::draw_rectangle(
        RGB8::new(0xff, 0, 0),
        &Vec2::new(0, 0),
        &Vec2::new(
            isize::try_from(Vram::resolution().x - 1).unwrap(),
            isize::try_from(Vram::resolution().y - 1).unwrap(),
        ),
    );

    if let Some(location) = info.location() {
        print_with_pos!(
            Vec2::new(0, 0),
            RGB8::new(0xff, 0xff, 0xff),
            "Panic in {} at ({}, {}):{}",
            location.file(),
            location.line(),
            location.column(),
            info.message().unwrap_or(&format_args!(""))
        );
    }

    loop {
        x86_64::instructions::hlt();
    }
}
