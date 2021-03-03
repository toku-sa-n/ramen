// SPDX-License-Identifier: GPL-3.0-or-later

// See P.114

use crate::{interrupt, x86_64::structures::idt::InterruptDescriptorTable};
use conquer_once::spin::Lazy;
use x86_64::PrivilegeLevel;

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    // SAFETY: Nested interrupts are not allowed.
    unsafe {
        idt[0x20]
            .set_handler_fn(interrupt::handler::h_20)
            .set_stack_index(0);
        idt[0x80]
            .set_handler_fn(interrupt::handler::h_80)
            .set_stack_index(0)
            .set_privilege_level(PrivilegeLevel::Ring3);
    }

    idt
});

pub fn init() {
    IDT.load();
}
