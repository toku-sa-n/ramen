// SPDX-License-Identifier: GPL-3.0-or-later

use common::constant::STACK_LOWER;
use x86_64::structures::tss::TaskStateSegment;

pub static TSS: TaskStateSegment = {
    let mut tss = TaskStateSegment::new();
    tss.privilege_stack_table[0] = STACK_LOWER;
    tss
};
