use super::mouse;
use crate::graphics;
use crate::graphics::screen;
use crate::print_with_pos;
use crate::x86_64::instructions::interrupts;

pub fn keyboard_data(vram: &graphics::Vram) -> () {
    let data: Option<u32> = super::KEY_QUEUE.lock().dequeue();

    interrupts::enable();

    let mut screen: graphics::screen::Screen = graphics::screen::Screen::new(vram);

    screen.draw_rectangle(
        graphics::RGB::new(0x008484),
        graphics::screen::Coord::new(0, 16),
        graphics::screen::Coord::new(15, 31),
    );

    if let Some(data) = data {
        print_with_pos!(
            vram,
            graphics::screen::Coord::new(0, 16),
            graphics::RGB::new(0xFFFFFF),
            "{:X}",
            data
        );
    }
}

pub fn mouse_data(
    mouse_device: &mut super::mouse::Device,
    mouse_cursor: &mut screen::MouseCursor,
    vram: &graphics::Vram,
) -> () {
    let data: Option<u32> = mouse::QUEUE.lock().dequeue();

    interrupts::enable();

    let mut screen: graphics::screen::Screen = graphics::screen::Screen::new(vram);

    screen.draw_rectangle(
        graphics::RGB::new(0x008484),
        graphics::screen::Coord::new(32, 16),
        graphics::screen::Coord::new(47, 31),
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
    mouse_cursor.print_coord(graphics::screen::Coord::new(16, 32));
}
