// SPDX-License-Identifier: GPL-3.0-or-later

use common::constant::RECUR_PML4_ADDR;
use conquer_once::spin::Lazy;
use spinning_top::Spinlock;
use x86_64::structures::paging::RecursivePageTable;

pub(crate) static PML4: Lazy<Spinlock<RecursivePageTable<'_>>> = Lazy::new(|| unsafe {
    Spinlock::new(
        (RecursivePageTable::new(&mut *(RECUR_PML4_ADDR.as_mut_ptr())))
            .expect("PML4 has no recursive entry."),
    )
});
