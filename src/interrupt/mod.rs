pub mod mouse;

use crate::asm;
use crate::queue;

extern crate lazy_static;

const PIC0_ICW1: u32 = 0x0020;
const PIC0_OCW2: u32 = 0x0020;
const PIC0_IMR: u32 = 0x0021;
const PIC0_ICW2: u32 = 0x0021;
const PIC0_ICW3: u32 = 0x0021;
const PIC0_ICW4: u32 = 0x0021;
const PIC1_ICW1: u32 = 0x00A0;
const PIC1_OCW2: u32 = 0x00A0;
const PIC1_IMR: u32 = 0x00A1;
const PIC1_ICW2: u32 = 0x00A1;
const PIC1_ICW3: u32 = 0x00A1;
const PIC1_ICW4: u32 = 0x00A1;

const PORT_KEYDATA: u32 = 0x0060;

const PORT_KEY_STATUS: u32 = 0x0064;
const PORT_KEY_CMD: u32 = 0x0064;
const KEY_STATUS_SEND_NOT_READY: u32 = 0x02;
const KEY_CMD_WRITE_MODE: u32 = 0x60;
const KEY_CMD_MODE: u32 = 0x47;
const KEY_CMD_SEND_TO_MOUSE: u32 = 0xD4;
const MOUSE_CMD_ENABLE: u32 = 0xF4;

lazy_static::lazy_static! {
    pub static ref KEY_QUEUE: spin::Mutex<queue::Queue> = spin::Mutex::new(queue::Queue::new());
}

// See P.128.
pub fn init_pic() -> () {
    asm::out8(PIC0_IMR, 0xFF);
    asm::out8(PIC1_IMR, 0xFF);

    asm::out8(PIC0_ICW1, 0x11);
    asm::out8(PIC0_ICW2, 0x20);
    asm::out8(PIC0_ICW3, 1 << 2);
    asm::out8(PIC0_ICW4, 0x01);

    asm::out8(PIC1_ICW1, 0x11);
    asm::out8(PIC1_ICW2, 0x28);
    asm::out8(PIC1_ICW3, 2);
    asm::out8(PIC1_ICW4, 0x01);

    asm::out8(PIC0_IMR, 0xFB);
    asm::out8(PIC1_IMR, 0xFF);
}

pub fn set_init_pic_bits() -> () {
    asm::out8(PIC0_IMR, 0xF9);
    asm::out8(PIC1_IMR, 0xEF);
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
    mouse::QUEUE.lock().enqueue(asm::in8(PORT_KEYDATA));
}
