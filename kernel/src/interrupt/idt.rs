// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::interrupt::handler::h_20, conquer_once::spin::Lazy,
    x86_64::structures::idt::InterruptDescriptorTable,
};

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    unsafe {
        idt[0x20].set_handler_fn(h_20).set_stack_index(0);
    }

    idt
});

pub(crate) fn init() {
    IDT.load();
}
