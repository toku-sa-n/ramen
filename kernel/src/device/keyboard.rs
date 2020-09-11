// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{graphics::screen::Screen, interrupt::KEY_QUEUE, print_with_pos},
    rgb::RGB8,
    vek::Vec2,
    x86_64::instructions::interrupts,
};

pub fn handler() {
    let data: Option<u32> = KEY_QUEUE.lock().pop_front();

    interrupts::enable();

    Screen::draw_rectangle(
        RGB8::new(0, 0x84, 0x84),
        &Vec2::new(0, 16),
        &Vec2::new(15, 31),
    );

    if let Some(data) = data {
        print_with_pos!(Vec2::new(0, 16), RGB8::new(0xff, 0xff, 0xff), "{:X}", data);
    }
}
