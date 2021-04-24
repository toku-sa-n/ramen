// SPDX-License-Identifier: GPL-3.0-or-later

use common::constant::INTERRUPT_STACK;
use spinning_top::Spinlock;
use x86_64::structures::tss::TaskStateSegment;

pub(crate) static TSS: Spinlock<TaskStateSegment> = {
    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[0] = INTERRUPT_STACK;
    Spinlock::new(tss)
};
