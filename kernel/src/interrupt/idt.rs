// SPDX-License-Identifier: GPL-3.0-or-later

use conquer_once::spin::Lazy;
use core::convert::TryInto;
use x86_64::{structures::idt::InterruptDescriptorTable, PrivilegeLevel, VirtAddr};

extern "C" {
    fn h_20_asm();
    fn h_80_asm();
    fn h_81_asm();
}

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    unsafe {
        idt[0x20]
            .set_handler_addr(handler_function_to_virt_addr(h_20_asm))
            .set_stack_index(0);

        idt[0x80]
            .set_handler_addr(handler_function_to_virt_addr(h_80_asm))
            .set_stack_index(0)
            .set_privilege_level(PrivilegeLevel::Ring3);

        idt[0x81]
            .set_handler_addr(handler_function_to_virt_addr(h_81_asm))
            .set_stack_index(0)
            .set_privilege_level(PrivilegeLevel::Ring3);
    }

    idt
});

pub(crate) fn init() {
    IDT.load();
}

fn handler_function_to_virt_addr(f: unsafe extern "C" fn()) -> VirtAddr {
    VirtAddr::new((f as usize).try_into().unwrap())
}
