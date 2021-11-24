// SPDX-License-Identifier: GPL-3.0-or-later

use {
    conquer_once::spin::Lazy,
    predefined_mmap::INTERRUPT_STACK,
    spinning_top::Spinlock,
    x86_64::{structures::tss::TaskStateSegment, VirtAddr},
};

static TSS: Lazy<Spinlock<TaskStateSegment>> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[0] = *INTERRUPT_STACK;
    Spinlock::new(tss)
});

pub(crate) fn get_ptr() -> *mut TaskStateSegment {
    TSS.data_ptr()
}

pub(crate) fn set_interrupt_stack(addr: VirtAddr) {
    TSS.lock().interrupt_stack_table[0] = addr;
}
