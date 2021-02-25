// SPDX-License-Identifier: GPL-3.0-or-later

// See P.114

use crate::{interrupt, x86_64::structures::idt::InterruptDescriptorTable};
use conquer_once::spin::Lazy;

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    // SAFETY: This operation is safe as the stack index 0 is allocated for the timer
    // interruption.
    unsafe {
        idt[0x20]
            .set_handler_fn(interrupt::handler::h_20)
            .set_stack_index(0);
    }

    idt
});

pub fn init() {
    IDT.load();
}
