// SPDX-License-Identifier: GPL-3.0-or-later

use super::mouse;
use crate::graphics::screen;
use crate::graphics::screen::Screen;
use crate::print_with_pos;
use crate::x86_64::instructions::interrupts;
use rgb::RGB8;
use vek::Vec2;

pub fn mouse_data(mouse_device: &mut super::mouse::Device, mouse_cursor: &mut screen::MouseCursor) {
    let data = mouse::QUEUE.lock().pop_front();

    interrupts::enable();

    Screen::draw_rectangle(
        RGB8::new(0, 0x84, 0x84),
        &Vec2::new(32, 16),
        &Vec2::new(47, 31),
    );

    if data == None {
        return;
    }

    mouse_device.put_data(data.unwrap());

    if mouse_device.data_available() {
        mouse_device.purse_data();
    }

    mouse_device.print_buf_data();
    mouse_cursor.draw_offset(mouse_device.get_speed());
    mouse_cursor.print_coord(Vec2::new(16, 32));
}
