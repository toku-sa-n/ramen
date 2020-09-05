// SPDX-License-Identifier: GPL-3.0-or-later

use crate::graphics::screen::Coord;
use crate::graphics::screen::Screen;
use crate::graphics::Vram;
use crate::graphics::RGB;
use crate::print_with_pos;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let mut screen = Screen;

    screen.draw_rectangle(
        RGB::new(0x00ff_0000),
        &Coord::new(0, 0),
        &Coord::new(
            Vram::resolution().x as isize - 1,
            Vram::resolution().y as isize - 1,
        ),
    );

    if let Some(location) = info.location() {
        print_with_pos!(
            Coord::new(0, 0),
            RGB::new(0x00ff_ffff),
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
