// SPDX-License-Identifier: GPL-3.0-or-later

use crate::graphics;
use crate::graphics::screen::Screen;
use crate::graphics::screen::TwoDimensionalVec;
use crate::queue;
use crate::x86_64::instructions::port::Port;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref QUEUE: spin::Mutex<queue::Queue<u8>> = spin::Mutex::new(queue::Queue::new(0));
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

    fn purse_data(data: u8) -> Self {
        Self {
            left: data & 0x01 != 0,
            right: data & 0x02 != 0,
            center: data & 0x04 != 0,
        }
    }
}

#[derive(PartialEq, Eq)]
enum DevicePhase {
    Init,
    NoData,
    OneData,
    TwoData,
    ThreeData,
}

pub struct Device {
    data_from_device: [u8; 3],
    phase: DevicePhase,

    speed: TwoDimensionalVec<i16>,

    buttons: MouseButtons,
}

impl Device {
    pub fn new() -> Self {
        Self {
            data_from_device: [0; 3],
            phase: DevicePhase::Init,
            speed: graphics::screen::TwoDimensionalVec::new(0, 0),
            buttons: MouseButtons::new(),
        }
    }

    pub fn enable() {
        super::wait_kbc_sendready();
        unsafe { Port::new(super::PORT_KEY_CMD).write(super::KEY_CMD_SEND_TO_MOUSE) };
        super::wait_kbc_sendready();
        unsafe { Port::new(super::PORT_KEYDATA).write(super::MOUSE_CMD_ENABLE) };
    }

    pub fn data_available(&self) -> bool {
        self.phase == DevicePhase::ThreeData
    }

    pub fn put_data(&mut self, data: u8) {
        match self.phase {
            DevicePhase::Init => {
                let is_correct_startup = data == 0xfa;
                if is_correct_startup {
                    self.phase = DevicePhase::NoData
                }
            }

            DevicePhase::NoData => {
                if Self::is_correct_first_byte_from_device(data) {
                    self.data_from_device[0] = data;
                    self.phase = DevicePhase::OneData;
                }
            }
            DevicePhase::OneData => {
                self.data_from_device[1] = data;
                self.phase = DevicePhase::TwoData;
            }
            DevicePhase::TwoData => {
                self.data_from_device[2] = data;
                self.phase = DevicePhase::ThreeData;
            }
            DevicePhase::ThreeData => {}
        }
    }

    // To sync phase, and data sent from mouse device
    fn is_correct_first_byte_from_device(data: u8) -> bool {
        data & 0xC8 == 0x08
    }

    fn clear_stack(&mut self) {
        self.phase = DevicePhase::NoData;
    }

    pub fn purse_data(&mut self) {
        self.buttons = MouseButtons::purse_data(self.data_from_device[0]);
        self.speed.x = self.data_from_device[1] as i16;
        self.speed.y = self.data_from_device[2] as i16;

        if self.data_from_device[0] & 0x10 != 0 {
            self.speed.x = (self.speed.x as u16 | 0xFF00) as i16;
        }

        if self.data_from_device[0] & 0x20 != 0 {
            self.speed.y = (self.speed.y as u16 | 0xFF00) as i16;
        }

        self.speed.y = -self.speed.y;

        self.clear_stack();
    }

    pub fn get_speed(&self) -> graphics::screen::Coord<isize> {
        graphics::screen::Coord::new(self.speed.x as isize, self.speed.y as isize)
    }

    pub fn print_buf_data(&mut self) {
        use crate::print_with_pos;

        Screen::draw_rectangle(
            graphics::RGB::new(0x0000_8484),
            &graphics::screen::Coord::new(32, 16),
            &graphics::screen::Coord::new(32 + 15 * 8 - 1, 31),
        );

        print_with_pos!(
            graphics::screen::Coord::new(32, 16),
            graphics::RGB::new(0x00FF_FFFF),
            "[{}{}{} {:4}{:4}]",
            if self.buttons.left { 'L' } else { 'l' },
            if self.buttons.center { 'C' } else { 'c' },
            if self.buttons.right { 'R' } else { 'r' },
            self.speed.x,
            self.speed.y
        );
    }
}
