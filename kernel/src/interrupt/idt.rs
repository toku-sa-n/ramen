// SPDX-License-Identifier: GPL-3.0-or-later

// See P.114

use crate::{interrupt, x86_64::structures::idt::InterruptDescriptorTable};
use conquer_once::spin::Lazy;

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    idt[0x00].set_handler_fn(interrupt::handler::h_00);
    idt[0x01].set_handler_fn(interrupt::handler::h_01);
    idt[0x02].set_handler_fn(interrupt::handler::h_02);
    idt[0x03].set_handler_fn(interrupt::handler::h_03);
    idt[0x04].set_handler_fn(interrupt::handler::h_04);
    idt[0x05].set_handler_fn(interrupt::handler::h_05);
    idt[0x06].set_handler_fn(interrupt::handler::h_06);
    idt[0x07].set_handler_fn(interrupt::handler::h_07);
    idt[0x09].set_handler_fn(interrupt::handler::h_09);
    idt[0x10].set_handler_fn(interrupt::handler::h_10);
    idt[0x13].set_handler_fn(interrupt::handler::h_13);
    idt[0x14].set_handler_fn(interrupt::handler::h_14);

    // SAFETY: This operation is safe as the stack index 0 is allocated for the timer
    // interruption.
    unsafe {
        idt[0x20]
            .set_handler_fn(interrupt::handler::h_20)
            .set_stack_index(0);
    }
    idt[0x21].set_handler_fn(interrupt::handler::h_21);
    idt[0x2c].set_handler_fn(interrupt::handler::h_2c);
    idt[0x40].set_handler_fn(interrupt::handler::h_40);

    idt
});

pub fn init() {
    IDT.load();
}
