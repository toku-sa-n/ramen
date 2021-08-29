// SPDX-License-Identifier: GPL-3.0-or-later

use super::handler;
use conquer_once::spin::Lazy;
use x86_64::{structures::idt::InterruptDescriptorTable, PrivilegeLevel};

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    unsafe {
        idt[0x20].set_handler_fn(handler::h_20).set_stack_index(0);

        idt[0x80]
            .set_handler_fn(handler::h_80)
            .set_stack_index(0)
            .set_privilege_level(PrivilegeLevel::Ring3);

        idt[0x81]
            .set_handler_fn(handler::h_81)
            .set_stack_index(0)
            .set_privilege_level(PrivilegeLevel::Ring3);
    }

    idt
});

pub(crate) fn init() {
    IDT.load();
}
