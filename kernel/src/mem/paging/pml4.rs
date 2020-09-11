// SPDX-License-Identifier: GPL-3.0-or-later

use common::constant::RECUR_PML4_ADDR;
use conquer_once::spin::Lazy;
use x86_64::structures::paging::{PageTable, RecursivePageTable};

static PML4: Lazy<RecursivePageTable> = Lazy::new(|| unsafe {
    (RecursivePageTable::new(&mut *(RECUR_PML4_ADDR.as_mut_ptr() as *mut PageTable))).unwrap()
});
