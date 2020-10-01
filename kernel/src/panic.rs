// SPDX-License-Identifier: GPL-3.0-or-later

use crate::graphics::screen::Screen;
use crate::graphics::Vram;
use rgb::RGB8;
use vek::Vec2;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("*************");
    error!("*   PANIC   *");
    error!("*************");
    if let Some(location) = info.location() {
        error!(
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
