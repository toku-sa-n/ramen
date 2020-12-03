// SPDX-License-Identifier: GPL-3.0-or-later

use crate::device::{keyboard, mouse};
use common::constant::PORT_KEY_DATA;
use x86_64::{instructions::port::Port, structures::idt};

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

pub extern "x86-interrupt" fn handler_00(_stack_frame: &mut idt::InterruptStackFrame) {
    panic!("Divide-by-zero Error!");
}

pub extern "x86-interrupt" fn handler_01(_stack_frame: &mut idt::InterruptStackFrame) {
    panic!("Debug exception!");
}

pub extern "x86-interrupt" fn handler_02(_stack_frame: &mut idt::InterruptStackFrame) {
    panic!("Non-maskable Interrupt!");
}

pub extern "x86-interrupt" fn handler_03(_stack_frame: &mut idt::InterruptStackFrame) {
    panic!("Breakpoint!");
}

pub extern "x86-interrupt" fn handler_04(_stack_frame: &mut idt::InterruptStackFrame) {
    panic!("Overflow!");
}

pub extern "x86-interrupt" fn handler_05(_stack_frame: &mut idt::InterruptStackFrame) {
    panic!("Bound Range Exceeded!");
}

pub extern "x86-interrupt" fn handler_06(_stack_frame: &mut idt::InterruptStackFrame) {
    panic!("Invalid Opcode!");
}

pub extern "x86-interrupt" fn handler_07(_stack_frame: &mut idt::InterruptStackFrame) {
    panic!("Device Not Available!");
}

pub extern "x86-interrupt" fn handler_09(_stack_frame: &mut idt::InterruptStackFrame) {
    panic!("Coprocessor Segment Overrun!");
}

pub extern "x86-interrupt" fn handler_10(_stack_frame: &mut idt::InterruptStackFrame) {
    panic!("x87 Floating-Point Exception");
}

pub extern "x86-interrupt" fn handler_13(_stack_frame: &mut idt::InterruptStackFrame) {
    panic!("SIMD Floating-Point Exception");
}

pub extern "x86-interrupt" fn handler_14(_stack_frame: &mut idt::InterruptStackFrame) {
    panic!("Virtualization Exception!");
}

pub extern "x86-interrupt" fn handler_20(_stack_frame: &mut idt::InterruptStackFrame) {
    unsafe {
        Port::new(PIC0_OCW2).write(0x60_u8);
    }
}

pub extern "x86-interrupt" fn handler_21(_stack_frame: &mut idt::InterruptStackFrame) {
    unsafe { Port::new(PIC0_OCW2).write(0x61_u8) };
    let mut port = PORT_KEY_DATA;
    keyboard::enqueue_scancode(unsafe { port.read() });
}

pub extern "x86-interrupt" fn handler_2c(_stack_frame: &mut idt::InterruptStackFrame) {
    unsafe {
        Port::new(PIC1_OCW2).write(0x64_u8);
        Port::new(PIC0_OCW2).write(0x62_u8);
    }
    let mut port = PORT_KEY_DATA;
    mouse::enqueue_packet(unsafe { port.read() });
}

pub extern "x86-interrupt" fn handler_40(_stack_frame: &mut idt::InterruptStackFrame) {
    info!("Interrupt from 0x40");
}
