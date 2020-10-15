// SPDX-License-Identifier: GPL-3.0-or-later

// See P.114

use crate::interrupt;
use crate::x86_64::structures::idt::InterruptDescriptorTable;
use conquer_once::spin::Lazy;

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    idt[0x00].set_handler_fn(interrupt::handler_00);
    idt[0x01].set_handler_fn(interrupt::handler_01);
    idt[0x02].set_handler_fn(interrupt::handler_02);
    idt[0x03].set_handler_fn(interrupt::handler_03);
    idt[0x04].set_handler_fn(interrupt::handler_04);
    idt[0x05].set_handler_fn(interrupt::handler_05);
    idt[0x06].set_handler_fn(interrupt::handler_06);
    idt[0x07].set_handler_fn(interrupt::handler_07);
    idt[0x09].set_handler_fn(interrupt::handler_09);
    idt[0x10].set_handler_fn(interrupt::handler_10);
    idt[0x13].set_handler_fn(interrupt::handler_13);
    idt[0x14].set_handler_fn(interrupt::handler_14);
    idt[0x20].set_handler_fn(interrupt::handler_20);
    idt[0x21].set_handler_fn(interrupt::handler_21);
    idt[0x2c].set_handler_fn(interrupt::handler_2c);
    idt[0x40].set_handler_fn(interrupt::handler_40);

    idt
});

pub fn init() {
    IDT.load();
}
