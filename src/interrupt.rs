use crate::asm;
use crate::queue;

extern crate lazy_static;

const PIC0_ICW1: i32 = 0x0020;
const PIC0_OCW2: i32 = 0x0020;
const PIC0_IMR: i32 = 0x0021;
const PIC0_ICW2: i32 = 0x0021;
const PIC0_ICW3: i32 = 0x0021;
const PIC0_ICW4: i32 = 0x0021;
const PIC1_ICW1: i32 = 0x00a0;
const PIC1_OCW2: i32 = 0x00a0;
const PIC1_IMR: i32 = 0x00a1;
const PIC1_ICW2: i32 = 0x00a1;
const PIC1_ICW3: i32 = 0x00a1;
const PIC1_ICW4: i32 = 0x00a1;

const PORT_KEYDATA: i32 = 0x0060;

const PORT_KEY_STATUS: i32 = 0x0064;
const PORT_KEY_CMD: i32 = 0x0064;
const KEY_STATUS_SEND_NOT_READY: i32 = 0x02;
const KEY_CMD_WRITE_MODE: i32 = 0x60;
const KEY_CMD_MODE: i32 = 0x47;
const KEY_CMD_SEND_TO_MOUSE: i32 = 0xd4;
const MOUSE_CMD_ENABLE: i32 = 0xf4;

lazy_static::lazy_static! {
    pub static ref KEY_QUEUE: spin::Mutex<queue::Queue> = spin::Mutex::new(queue::Queue::new());
    pub static ref MOUSE_QUEUE:spin::Mutex<queue::Queue> = spin::Mutex::new(queue::Queue::new());
}

use crate::graphics;

pub struct MouseDevice {
    data_from_device: [i32; 3],
    phase: i32,

    x_speed: i32,
    y_speed: i32,
    button_left: bool,
    button_center: bool,
    button_right: bool,
}

impl MouseDevice {
    pub fn new() -> Self {
        Self {
            data_from_device: [0; 3],
            phase: 0,
            x_speed: 0,
            y_speed: 0,
            button_left: false,
            button_center: false,
            button_right: false,
        }
    }

    pub fn enable(&self) -> () {
        wait_kbc_sendready();
        asm::out8(PORT_KEY_CMD, KEY_CMD_SEND_TO_MOUSE);
        wait_kbc_sendready();
        asm::out8(PORT_KEYDATA, MOUSE_CMD_ENABLE);
    }

    // Return true if three bytes data are sent.
    // Otherwise return false.
    pub fn put_data(&mut self, data: i32) -> bool {
        match self.phase {
            0 => {
                if data == 0xfa {
                    self.phase = 1;
                }
                false
            }
            1 => {
                if Self::is_correct_first_byte_from_device(data) {
                    self.data_from_device[0] = data;
                    self.phase = 2;
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
        self.button_left = self.data_from_device[0] & 0x01 != 0;
        self.button_right = self.data_from_device[0] & 0x02 != 0;
        self.button_center = self.data_from_device[0] & 0x04 != 0;

        self.x_speed = self.data_from_device[1];
        self.y_speed = self.data_from_device[2];

        if self.data_from_device[0] & 0x10 != 0 {
            // -256 = 0xffffff00
            self.x_speed |= -256;
        }

        if self.data_from_device[0] & 0x20 != 0 {
            self.y_speed |= -256;
        }

        self.y_speed = -self.y_speed;
    }

    // To sync phase, and data sent from mouse device
    fn is_correct_first_byte_from_device(data: i32) -> bool {
        data & 0xc8 == 0x08
    }

    pub fn print_buf_data(&self) -> () {
        use crate::print_with_pos;
        let screen: graphics::screen::Screen = graphics::screen::Screen::new(graphics::Vram::new());

        screen.draw_rectangle(
            graphics::screen::ColorIndex::Rgb008484,
            graphics::screen::Coord::new(32, 16),
            graphics::screen::Coord::new(32 + 15 * 8 - 1, 31),
        );

        print_with_pos!(
            graphics::screen::Coord::new(32, 16),
            graphics::screen::ColorIndex::RgbFFFFFF,
            "[{}{}{} {:4}{:4}]",
            if self.button_left { 'L' } else { 'l' },
            if self.button_center { 'C' } else { 'c' },
            if self.button_right { 'R' } else { 'r' },
            self.x_speed,
            self.y_speed
        );
    }
}

// See P.128.
pub fn init_pic() -> () {
    asm::out8(PIC0_IMR, 0xff);
    asm::out8(PIC1_IMR, 0xff);

    asm::out8(PIC0_ICW1, 0x11);
    asm::out8(PIC0_ICW2, 0x20);
    asm::out8(PIC0_ICW3, 1 << 2);
    asm::out8(PIC0_ICW4, 0x01);

    asm::out8(PIC1_ICW1, 0x11);
    asm::out8(PIC1_ICW2, 0x28);
    asm::out8(PIC1_ICW3, 2);
    asm::out8(PIC1_ICW4, 0x01);

    asm::out8(PIC0_IMR, 0xfb);
    asm::out8(PIC1_IMR, 0xff);
}

pub fn set_init_pic_bits() -> () {
    asm::out8(PIC0_IMR, 0xf9);
    asm::out8(PIC1_IMR, 0xef);
}

fn wait_kbc_sendready() -> () {
    loop {
        if asm::in8(PORT_KEY_STATUS) & KEY_STATUS_SEND_NOT_READY == 0 {
            break;
        }
    }
}

pub fn init_keyboard() -> () {
    wait_kbc_sendready();
    asm::out8(PORT_KEY_CMD, KEY_CMD_WRITE_MODE);
    wait_kbc_sendready();
    asm::out8(PORT_KEYDATA, KEY_CMD_MODE);
}

pub extern "C" fn interrupt_handler_21() -> () {
    asm::out8(PIC0_OCW2, 0x61);
    KEY_QUEUE.lock().enqueue(asm::in8(PORT_KEYDATA));
}

pub extern "C" fn interrupt_handler_2c() -> () {
    asm::out8(PIC1_OCW2, 0x64);
    asm::out8(PIC0_OCW2, 0x62);
    MOUSE_QUEUE.lock().enqueue(asm::in8(PORT_KEYDATA));
}
