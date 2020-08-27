pub mod handler;
pub mod mouse;

use crate::queue;
use crate::x86_64::instructions::port::Port;
use crate::x86_64::structures::idt;

extern crate lazy_static;

const PIC0_ICW1: u16 = 0x0020;
const PIC0_OCW2: u16 = 0x0020;
const PIC0_IMR: u16 = 0x0021;
const PIC0_ICW2: u16 = 0x0021;
const PIC0_ICW3: u16 = 0x0021;
const PIC0_ICW4: u16 = 0x0021;
const PIC1_ICW1: u16 = 0x00A0;
const PIC1_OCW2: u16 = 0x00A0;
const PIC1_IMR: u16 = 0x00A1;
const PIC1_ICW2: u16 = 0x00A1;
const PIC1_ICW3: u16 = 0x00A1;
const PIC1_ICW4: u16 = 0x00A1;

const PORT_KEYDATA: u16 = 0x0060;

const PORT_KEY_STATUS: u16 = 0x0064;
const PORT_KEY_CMD: u16 = 0x0064;
const KEY_STATUS_SEND_NOT_READY: u8 = 0x02;
const KEY_CMD_WRITE_MODE: u8 = 0x60;
const KEY_CMD_MODE: u8 = 0x47;
const KEY_CMD_SEND_TO_MOUSE: u8 = 0xD4;
const MOUSE_CMD_ENABLE: u8 = 0xF4;

lazy_static::lazy_static! {
    pub static ref KEY_QUEUE: spin::Mutex<queue::Queue> = spin::Mutex::new(queue::Queue::new());
}

// See P.128.
pub fn init_pic() -> () {
    unsafe {
        Port::new(PIC0_IMR).write(0xFF as u8);
        Port::new(PIC1_IMR).write(0xFF as u8);

        Port::new(PIC0_ICW1).write(0x11 as u8);
        Port::new(PIC0_ICW2).write(0x20 as u8);
        Port::new(PIC0_ICW3).write((1 << 2) as u8);
        Port::new(PIC0_ICW4).write(0x01 as u8);

        Port::new(PIC1_ICW1).write(0x11 as u8);
        Port::new(PIC1_ICW2).write(0x28 as u8);
        Port::new(PIC1_ICW3).write(2 as u8);
        Port::new(PIC1_ICW4).write(0x01 as u8);

        Port::new(PIC0_IMR).write(0xFB as u8);
        Port::new(PIC1_IMR).write(0xFF as u8);
    }
}

pub fn set_init_pic_bits() -> () {
    unsafe {
        Port::new(PIC0_IMR).write(0xF9 as u8);
        Port::new(PIC1_IMR).write(0xEF as u8);
    }
}

pub fn init_keyboard() -> () {
    wait_kbc_sendready();
    unsafe { Port::new(PORT_KEY_CMD).write(KEY_CMD_WRITE_MODE as u8) };
    wait_kbc_sendready();
    unsafe { Port::new(PORT_KEYDATA).write(KEY_CMD_MODE as u8) };
}

fn wait_kbc_sendready() -> () {
    loop {
        if unsafe { Port::<u8>::new(PORT_KEY_STATUS).read() } & KEY_STATUS_SEND_NOT_READY == 0 {
            break;
        }
    }
}

pub extern "x86-interrupt" fn interrupt_handler_21(
    _stack_frame: &mut idt::InterruptStackFrame,
) -> () {
    unsafe { Port::new(PIC0_OCW2).write(0x61 as u8) };
    KEY_QUEUE
        .lock()
        .enqueue(unsafe { Port::<u8>::new(PORT_KEYDATA).read() as u32 });
}

pub extern "x86-interrupt" fn interrupt_handler_2c(
    _stack_frame: &mut idt::InterruptStackFrame,
) -> () {
    unsafe {
        Port::new(PIC1_OCW2).write(0x64 as u8);
        Port::new(PIC0_OCW2).write(0x62 as u8);
    }
    mouse::QUEUE
        .lock()
        .enqueue(unsafe { Port::<u8>::new(PORT_KEYDATA).read() as u32 });
}
