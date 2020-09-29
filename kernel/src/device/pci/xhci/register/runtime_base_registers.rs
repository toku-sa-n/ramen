// SPDX-License-Identifier: GPL-3.0-or-later

#[repr(C)]
pub struct EventRingSegmentTable {
    base_address: u64,
    size: u64,
}
