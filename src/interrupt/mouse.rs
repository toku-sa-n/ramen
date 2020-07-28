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

pub struct Device<'a> {
    data_from_device: [u32; 3],
    phase: u32,

    speed: graphics::screen::TwoDimensionalVec<i32>,

    buttons: MouseButtons,

    vram: &'a graphics::Vram,
}

impl<'a> Device<'a> {
    pub fn new(vram: &'a graphics::Vram) -> Self {
        Self {
            data_from_device: [0; 3],
            phase: 0,
            speed: graphics::screen::TwoDimensionalVec::new(0, 0),
            buttons: MouseButtons::new(),
            vram,
        }
    }

    pub fn enable(&self) -> () {
        super::wait_kbc_sendready();
        asm::out8(super::PORT_KEY_CMD, super::KEY_CMD_SEND_TO_MOUSE as u8);
        super::wait_kbc_sendready();
        asm::out8(super::PORT_KEYDATA, super::MOUSE_CMD_ENABLE as u8);
    }

    // Return true if three bytes data are sent.
    // Otherwise return false.
    pub fn put_data(&mut self, data: u32) -> bool {
        match self.phase {
            0 => {
                self.phase = if data == 0xfa { 1 } else { 0 };
                false
            }

            1 => {
                if Self::is_correct_first_byte_from_device(data) {
                    self.phase = 2;
                    self.data_from_device[0] = data;
                }
                false
            }
            2 => {
                self.data_from_device[1] = data;
                self.phase = 3;
                false
            }
            3 => {
                self.data_from_device[2] = data;
                self.phase = 1;

                self.purse_data();

                true
            }
            _ => {
                self.phase = 1;
                true
            }
        }
    }

    fn purse_data(&mut self) -> () {
        self.buttons = MouseButtons::purse_data(self.data_from_device[0]);
        self.speed.x = self.data_from_device[1] as i32;
        self.speed.y = self.data_from_device[2] as i32;

        if self.data_from_device[0] & 0x10 != 0 {
            self.speed.x = (self.speed.x as u32 | 0xFFFFFF00) as i32;
        }

        if self.data_from_device[0] & 0x20 != 0 {
            self.speed.y = (self.speed.y as u32 | 0xFFFFFF00) as i32;
        }

        self.speed.y = -self.speed.y;
    }

    // To sync phase, and data sent from mouse device
    fn is_correct_first_byte_from_device(data: u32) -> bool {
        data & 0xC8 == 0x08
    }

    pub fn get_speed(&self) -> graphics::screen::Coord<isize> {
        graphics::screen::Coord::new(self.speed.x as isize, self.speed.y as isize)
    }

    pub fn print_buf_data(&mut self) -> () {
        use crate::print_with_pos;
        let mut screen: graphics::screen::Screen = graphics::screen::Screen::new(self.vram);

        screen.draw_rectangle(
            graphics::RGB::new(0x008484),
            graphics::screen::Coord::new(32, 16),
            graphics::screen::Coord::new(32 + 15 * 8 - 1, 31),
        );

        print_with_pos!(
            self.vram,
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
