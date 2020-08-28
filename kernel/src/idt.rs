// SPDX-License-Identifier: GPL-3.0-or-later
// See P.114

use crate::interrupt::interrupt_handler_21;
use crate::interrupt::interrupt_handler_2c;
use crate::x86_64::structures::idt;
use lazy_static::lazy_static;

lazy_static! {
    static ref IDT: idt::InterruptDescriptorTable = {
        let mut idt = idt::InterruptDescriptorTable::new();
        idt[0x21].set_handler_fn(interrupt_handler_21);
        idt[0x2c].set_handler_fn(interrupt_handler_2c);

        idt
    };
}

pub fn init() -> () {
    IDT.load();
}
