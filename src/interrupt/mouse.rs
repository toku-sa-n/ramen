use crate::asm;
use crate::graphics;
use crate::queue;

extern crate lazy_static;

lazy_static::lazy_static! {
    pub static ref QUEUE:spin::Mutex<queue::Queue> = spin::Mutex::new(queue::Queue::new());
}

struct MouseButtons {
    left: bool,
    center: bool,
    right: bool,
}

impl MouseButtons {
    fn new() -> Self {
        Self {
            left: false,
            right: false,
            center: false,
        }
    }

    fn purse_data(data: u32) -> Self {
        Self {
            left: data & 0x01 != 0,
            right: data & 0x02 != 0,
            center: data & 0x04 != 0,
        }
    }
}

pub struct Device {
    data_from_device: [u32; 3],
    phase: u32,

    speed: graphics::screen::TwoDimensionalVec<i32>,

    buttons: MouseButtons,
}

impl Device {
    pub fn new() -> Self {
        Self {
            data_from_device: [0; 3],
            phase: 0,
            speed: graphics::screen::TwoDimensionalVec::new(0, 0),
            buttons: MouseButtons::new(),
        }
    }

    pub fn enable(&self) -> () {
        super::wait_kbc_sendready();
        asm::out8(super::PORT_KEY_CMD, super::KEY_CMD_SEND_TO_MOUSE);
        super::wait_kbc_sendready();
        asm::out8(super::PORT_KEYDATA, super::MOUSE_CMD_ENABLE);
    }

    // Return true if three bytes data are sent.
    // Otherwise return false.
    pub fn put_data(self, data: u32) -> (bool, Self) {
        match self.phase {
            0 => (
                false,
                Self {
                    phase: if data == 0xfa { 1 } else { 0 },
                    ..self
                },
            ),

            1 => {
                let mut new_self = self;
                if Self::is_correct_first_byte_from_device(data) {
                    new_self.phase = 2;
                    new_self.data_from_device[0] = data;
                }
                (false, new_self)
            }
            2 => {
                let mut new_self = self;
                new_self.data_from_device[1] = data;
                new_self.phase = 3;
                (false, new_self)
            }
            3 => {
                let mut new_self = self;

                new_self.data_from_device[2] = data;
                new_self.phase = 1;

                (true, new_self.purse_data())
            }
            _ => (true, Self { phase: 1, ..self }),
        }
    }

    fn purse_data(self) -> Self {
        let mut new_self = self;
        new_self.buttons = MouseButtons::purse_data(new_self.data_from_device[0]);
        new_self.speed.x = new_self.data_from_device[1] as i32;
        new_self.speed.y = new_self.data_from_device[2] as i32;

        if new_self.data_from_device[0] & 0x10 != 0 {
            new_self.speed.x = (new_self.speed.x as u32 | 0xFFFFFF00) as i32;
        }

        if new_self.data_from_device[0] & 0x20 != 0 {
            new_self.speed.y = (new_self.speed.y as u32 | 0xFFFFFF00) as i32;
        }

        new_self.speed.y = -new_self.speed.y;

        new_self
    }

    // To sync phase, and data sent from mouse device
    fn is_correct_first_byte_from_device(data: u32) -> bool {
        data & 0xc8 == 0x08
    }

    pub fn get_speed(&self) -> graphics::screen::Coord<isize> {
        graphics::screen::Coord::new(self.speed.x as isize, self.speed.y as isize)
    }

    pub fn print_buf_data(&self) -> () {
        use crate::print_with_pos;
        let mut screen: graphics::screen::Screen =
            graphics::screen::Screen::new(graphics::Vram::new());

        screen.draw_rectangle(
            graphics::RGB::new(0x008484),
            graphics::screen::Coord::new(32, 16),
            graphics::screen::Coord::new(32 + 15 * 8 - 1, 31),
        );

        print_with_pos!(
            graphics::screen::Coord::new(32, 16),
            graphics::RGB::new(0xFFFFFF),
            "[{}{}{} {:4}{:4}]",
            if self.buttons.left { 'L' } else { 'l' },
            if self.buttons.center { 'C' } else { 'c' },
            if self.buttons.right { 'R' } else { 'r' },
            self.speed.x,
            self.speed.y
        );
    }
}
