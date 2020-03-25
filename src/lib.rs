#![no_std]
#![feature(asm)]
#![feature(start)]
#![feature(naked_functions)]

mod asm;
mod descriptor_table;
mod interrupt;
mod queue;

#[macro_use]
mod graphics;

#[no_mangle]
#[start]
pub fn os_main() {
    let mouse_device: interrupt::mouse::Device = interrupt::mouse::Device::new();
    let mut mouse_cursor: graphics::screen::MouseCursor = graphics::screen::MouseCursor::new(
        graphics::RGB::new(0x008484),
        graphics::screen::MOUSE_GRAPHIC,
        graphics::Vram::new(),
    );

    mouse_cursor = initialization(&mouse_device, mouse_cursor);

    main_loop(mouse_device, mouse_cursor)
}

fn initialization(
    mouse_device: &interrupt::mouse::Device,
    mouse_cursor: graphics::screen::MouseCursor,
) -> graphics::screen::MouseCursor {
    descriptor_table::init();
    interrupt::init_pic();
    // Temporarily disable interruption to see whether desktop is drawn successfully or not.
    // asm::sti();
    let vram: graphics::Vram = graphics::Vram::new();

    graphics::screen::draw_desktop(&vram);

    print_with_pos!(
        graphics::screen::Coord::new(16, 64),
        graphics::RGB::new(0xFFFFFF),
        "x_len = {}",
        vram.x_len
    );

    interrupt::set_init_pic_bits();
    interrupt::init_keyboard();
    mouse_device.enable();

    mouse_cursor.draw_offset(graphics::screen::Coord::new(300, 300))
}

fn main_loop(
    mut mouse_device: interrupt::mouse::Device,
    mut mouse_cursor: graphics::screen::MouseCursor,
) -> () {
    loop {
        asm::cli();
        if interrupt::KEY_QUEUE.lock().size() != 0 {
            handle_keyboard_data();
        } else if interrupt::mouse::QUEUE.lock().size() != 0 {
            let (new_mouse_device, new_mouse_cursor) =
                handle_mouse_data(mouse_device, mouse_cursor);
            mouse_device = new_mouse_device;
            mouse_cursor = new_mouse_cursor;
        } else {
            asm::stihlt();
        }
    }
}

fn handle_keyboard_data() -> () {
    let data: Option<u32> = interrupt::KEY_QUEUE.lock().dequeue();

    asm::sti();

    let mut screen: graphics::screen::Screen = graphics::screen::Screen::new(graphics::Vram::new());

    screen.draw_rectangle(
        graphics::RGB::new(0x008484),
        graphics::screen::Coord::new(0, 16),
        graphics::screen::Coord::new(15, 31),
    );

    if let Some(data) = data {
        print_with_pos!(
            graphics::screen::Coord::new(0, 16),
            graphics::RGB::new(0xFFFFFF),
            "{:X}",
            data
        );
    }
}

fn handle_mouse_data(
    mouse_device: interrupt::mouse::Device,
    mouse_cursor: graphics::screen::MouseCursor,
) -> (interrupt::mouse::Device, graphics::screen::MouseCursor) {
    let data: Option<u32> = interrupt::mouse::QUEUE.lock().dequeue();

    asm::sti();

    let mut screen: graphics::screen::Screen = graphics::screen::Screen::new(graphics::Vram::new());

    screen.draw_rectangle(
        graphics::RGB::new(0x008484),
        graphics::screen::Coord::new(32, 16),
        graphics::screen::Coord::new(47, 31),
    );

    if data == None {
        return (mouse_device, mouse_cursor);
    }

    let (result, new_mouse_device) = mouse_device.put_data(data.unwrap());

    if !result {
        return (new_mouse_device, mouse_cursor);
    }
    new_mouse_device.print_buf_data();
    let new_mouse_cursor: graphics::screen::MouseCursor =
        mouse_cursor.draw_offset(new_mouse_device.get_speed());
    new_mouse_cursor.print_coord(graphics::screen::Coord::new(16, 32));

    (new_mouse_device, new_mouse_cursor)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        asm::hlt()
    }
}
