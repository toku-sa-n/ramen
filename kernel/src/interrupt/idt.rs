// SPDX-License-Identifier: GPL-3.0-or-later

// See P.114

use crate::{interrupt, x86_64::structures::idt::InterruptDescriptorTable};
use conquer_once::spin::Lazy;

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    idt[0x20].set_handler_fn(interrupt::handler::h_20);
    idt[0x21].set_handler_fn(interrupt::handler::h_21);
    idt[0x2c].set_handler_fn(interrupt::handler::h_2c);

    idt
});

pub fn init() {
    IDT.load();
}
