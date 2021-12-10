// SPDX-License-Identifier: GPL-3.0-or-later

use {spinning_top::Spinlock, x86_64::structures::tss::TaskStateSegment};

static TSS: Spinlock<TaskStateSegment> = Spinlock::new(TaskStateSegment::new());

pub(crate) fn get_ptr() -> *mut TaskStateSegment {
    TSS.data_ptr()
}
