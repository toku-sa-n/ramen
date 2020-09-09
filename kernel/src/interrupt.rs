// SPDX-License-Identifier: GPL-3.0-or-later

pub mod handler;
pub mod mouse;

use crate::queue;
use crate::x86_64::instructions::port::Port;
use crate::x86_64::structures::idt;

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

use conquer_once::spin::Lazy;
use spinning_top::Spinlock;

pub static KEY_QUEUE: Lazy<Spinlock<queue::Queue<u32>>> =
    Lazy::new(|| Spinlock::new(queue::Queue::new()));

// See P.128.
pub fn init_pic() {
    enable_interrupts_from_only_mouse_and_keyboard();
    enable_edge_trigger_mode();
    set_irq_receiver();
    set_connection();
    enable_nonbuffer_mode();
}

fn enable_interrupts_from_only_mouse_and_keyboard() {
    unsafe {
        Port::new(PIC0_IMR).write(0xFF as u8);
        Port::new(PIC1_IMR).write(0xFF as u8);
        Port::new(PIC0_IMR).write(0xFB as u8);
        Port::new(PIC1_IMR).write(0xFF as u8);
    }
}

fn enable_edge_trigger_mode() {
    unsafe {
        Port::new(PIC0_ICW1).write(0x11 as u8);
        Port::new(PIC1_ICW1).write(0x11 as u8);
    }
}

fn set_irq_receiver() {
    unsafe {
        Port::new(PIC0_ICW2).write(0x20 as u8);
        Port::new(PIC1_ICW2).write(0x28 as u8);
    }
}

fn set_connection() {
    unsafe {
        Port::new(PIC0_ICW3).write(4_u8);
        Port::new(PIC1_ICW3).write(2_u8);
    }
}

fn enable_nonbuffer_mode() {
    unsafe {
        Port::new(PIC0_ICW4).write(0x01 as u8);
        Port::new(PIC1_ICW4).write(0x01 as u8);
    }
}

pub fn set_init_pic_bits() {
    unsafe {
        Port::new(PIC0_IMR).write(0xF9 as u8);
        Port::new(PIC1_IMR).write(0xEF as u8);
    }
}

pub fn init_keyboard() {
    wait_kbc_sendready();
    unsafe { Port::new(PORT_KEY_CMD).write(KEY_CMD_WRITE_MODE as u8) };
    wait_kbc_sendready();
    unsafe { Port::new(PORT_KEYDATA).write(KEY_CMD_MODE as u8) };
}

fn wait_kbc_sendready() {
    loop {
        if unsafe { Port::<u8>::new(PORT_KEY_STATUS).read() } & KEY_STATUS_SEND_NOT_READY == 0 {
            break;
        }
    }
}

pub extern "x86-interrupt" fn handler_21(_stack_frame: &mut idt::InterruptStackFrame) {
    unsafe { Port::new(PIC0_OCW2).write(0x61 as u8) };
    KEY_QUEUE
        .lock()
        .enqueue(unsafe { u32::from(Port::<u8>::new(PORT_KEYDATA).read()) });
}

pub extern "x86-interrupt" fn handler_2c(_stack_frame: &mut idt::InterruptStackFrame) {
    unsafe {
        Port::new(PIC1_OCW2).write(0x64 as u8);
        Port::new(PIC0_OCW2).write(0x62 as u8);
    }
    mouse::QUEUE
        .lock()
        .enqueue(unsafe { Port::<u8>::new(PORT_KEYDATA).read() });
}
