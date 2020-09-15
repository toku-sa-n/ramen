// SPDX-License-Identifier: GPL-3.0-or-later

use {
    common::constant::RECUR_PML4_ADDR,
    conquer_once::spin::Lazy,
    spinning_top::Spinlock,
    x86_64::structures::paging::{PageTable, RecursivePageTable},
};

pub static PML4: Lazy<Spinlock<RecursivePageTable>> = Lazy::new(|| unsafe {
    Spinlock::new(
        (RecursivePageTable::new(&mut *(RECUR_PML4_ADDR.as_mut_ptr() as *mut PageTable)))
            .expect("PML4 has no recursive entry."),
    )
});
