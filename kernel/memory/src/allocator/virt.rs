// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryFrom;
use os_units::{Bytes, NumOfPages};
use x86_64::{
    structures::paging::{PageSize, Size4KiB},
    VirtAddr,
};

use crate::paging;

const BYTES_AVAILABLE_RAM: Bytes = Bytes::new(0x1_0000_0000_0000);

pub(crate) fn search_free_addr(n: NumOfPages<Size4KiB>) -> Option<VirtAddr> {
    let mut c = 0;
    let mut start = None;
    for a in (0..BYTES_AVAILABLE_RAM.as_usize()).step_by(usize::try_from(Size4KiB::SIZE).unwrap()) {
        let a = VirtAddr::new(a as _);
        if available(a) {
            if start.is_none() {
                start = Some(a);
            }

            c += 1;

            if c >= n.as_usize() {
                return start;
            }
        } else {
            c = 0;
            start = None;
        }
    }

    None
}

fn available(a: VirtAddr) -> bool {
    paging::translate(a).is_none() && !a.is_null()
}
