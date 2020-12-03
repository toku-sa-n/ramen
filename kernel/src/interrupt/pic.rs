use x86_64::instructions::port::Port;

use super::{
    PIC0_ICW1, PIC0_ICW2, PIC0_ICW3, PIC0_ICW4, PIC0_IMR, PIC1_ICW1, PIC1_ICW2, PIC1_ICW3,
    PIC1_ICW4, PIC1_IMR,
};

// SPDX-License-Identifier: GPL-3.0-or-later

// See P.128.
pub fn init() {
    enable_interrupts_from_only_mouse_and_keyboard();
    enable_edge_trigger_mode();
    set_irq_receiver();
    set_connection();
    enable_nonbuffer_mode();
}

fn enable_interrupts_from_only_mouse_and_keyboard() {
    unsafe {
        Port::new(PIC0_IMR).write(0xFF_u8);
        Port::new(PIC1_IMR).write(0xFF_u8);
        Port::new(PIC0_IMR).write(0xFB_u8);
        Port::new(PIC1_IMR).write(0xFF_u8);
    }
}

fn enable_edge_trigger_mode() {
    unsafe {
        Port::new(PIC0_ICW1).write(0x11_u8);
        Port::new(PIC1_ICW1).write(0x11_u8);
    }
}

fn set_irq_receiver() {
    unsafe {
        Port::new(PIC0_ICW2).write(0x20_u8);
        Port::new(PIC1_ICW2).write(0x28_u8);
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
        Port::new(PIC0_ICW4).write(0x01_u8);
        Port::new(PIC1_ICW4).write(0x01_u8);
    }
}

pub fn set_init_pic_bits() {
    unsafe {
        Port::new(PIC0_IMR).write(0xF9_u8);
        Port::new(PIC1_IMR).write(0xEF_u8);
    }
}
