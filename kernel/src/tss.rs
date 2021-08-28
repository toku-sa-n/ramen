// SPDX-License-Identifier: GPL-3.0-or-later

use common::constant::INTERRUPT_STACK;
use conquer_once::spin::Lazy;
use spinning_top::Spinlock;
use x86_64::structures::tss::TaskStateSegment;

pub(crate) static TSS: Lazy<Spinlock<TaskStateSegment>> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[0] = *INTERRUPT_STACK;
    Spinlock::new(tss)
});
