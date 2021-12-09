// SPDX-License-Identifier: GPL-3.0-or-later

use {
    spinning_top::Spinlock,
    x86_64::{structures::tss::TaskStateSegment, VirtAddr},
};

static TSS: Spinlock<TaskStateSegment> = Spinlock::new(TaskStateSegment::new());

pub(crate) fn get_ptr() -> *mut TaskStateSegment {
    TSS.data_ptr()
}

pub(crate) fn set_interrupt_stack(addr: VirtAddr) {
    TSS.lock().interrupt_stack_table[0] = addr;
}
