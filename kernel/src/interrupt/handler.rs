// SPDX-License-Identifier: GPL-3.0-or-later

use common::constant::PORT_KEY_DATA;
use x86_64::instructions::port::Port;

use crate::device::{keyboard, mouse};

use super::{apic, PIC0_OCW2};

pub extern "x86-interrupt" fn h_00(
    _stack_frame: &mut x86_64::structures::idt::InterruptStackFrame,
) {
    panic!("Divide-by-zero Error!");
}

pub extern "x86-interrupt" fn h_01(
    _stack_frame: &mut x86_64::structures::idt::InterruptStackFrame,
) {
    panic!("Debug exception!");
}

pub extern "x86-interrupt" fn h_02(
    _stack_frame: &mut x86_64::structures::idt::InterruptStackFrame,
) {
    panic!("Non-maskable Interrupt!");
}

pub extern "x86-interrupt" fn h_03(
    _stack_frame: &mut x86_64::structures::idt::InterruptStackFrame,
) {
    panic!("Breakpoint!");
}

pub extern "x86-interrupt" fn h_04(
    _stack_frame: &mut x86_64::structures::idt::InterruptStackFrame,
) {
    panic!("Overflow!");
}

pub extern "x86-interrupt" fn h_05(
    _stack_frame: &mut x86_64::structures::idt::InterruptStackFrame,
) {
    panic!("Bound Range Exceeded!");
}

pub extern "x86-interrupt" fn h_06(
    _stack_frame: &mut x86_64::structures::idt::InterruptStackFrame,
) {
    panic!("Invalid Opcode!");
}

pub extern "x86-interrupt" fn h_07(
    _stack_frame: &mut x86_64::structures::idt::InterruptStackFrame,
) {
    panic!("Device Not Available!");
}

pub extern "x86-interrupt" fn h_09(
    _stack_frame: &mut x86_64::structures::idt::InterruptStackFrame,
) {
    panic!("Coprocessor Segment Overrun!");
}

pub extern "x86-interrupt" fn h_10(
    _stack_frame: &mut x86_64::structures::idt::InterruptStackFrame,
) {
    panic!("x87 Floating-Point Exception");
}

pub extern "x86-interrupt" fn h_13(
    _stack_frame: &mut x86_64::structures::idt::InterruptStackFrame,
) {
    panic!("SIMD Floating-Point Exception");
}

pub extern "x86-interrupt" fn h_14(
    _stack_frame: &mut x86_64::structures::idt::InterruptStackFrame,
) {
    panic!("Virtualization Exception!");
}

pub extern "x86-interrupt" fn h_20(
    _stack_frame: &mut x86_64::structures::idt::InterruptStackFrame,
) {
    unsafe {
        Port::new(PIC0_OCW2).write(0x60_u8);
    }
}

pub extern "x86-interrupt" fn h_21(
    _stack_frame: &mut x86_64::structures::idt::InterruptStackFrame,
) {
    apic::local::end_of_interrupt();
    let mut port = PORT_KEY_DATA;
    keyboard::enqueue_scancode(unsafe { port.read() });
}

pub extern "x86-interrupt" fn h_2c(
    _stack_frame: &mut x86_64::structures::idt::InterruptStackFrame,
) {
    apic::local::end_of_interrupt();
    let mut port = PORT_KEY_DATA;
    mouse::enqueue_packet(unsafe { port.read() });
}

pub extern "x86-interrupt" fn h_40(
    _stack_frame: &mut x86_64::structures::idt::InterruptStackFrame,
) {
    info!("Interrupt from 0x40");
}
