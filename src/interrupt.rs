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

pub fn enable_mouse() -> () {
    wait_kbc_sendready();
    asm::out8(PORT_KEY_CMD, KEY_CMD_SEND_TO_MOUSE);
    wait_kbc_sendready();
    asm::out8(PORT_KEYDATA, MOUSE_CMD_ENABLE);
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
