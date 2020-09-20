// SPDX-License-Identifier: GPL-3.0-or-later

pub mod pml4;

use {common::constant::RECUR_PML4_ADDR, x86_64::structures::paging::PageTable};

pub fn mark_pages_as_unused() {
    let page_table = unsafe { &mut *(RECUR_PML4_ADDR.as_mut_ptr() as *mut PageTable) };

    // Entry 510 and 511 are used by kernel.
    for i in 0..510 {
        page_table[i].set_unused();
    }
}
