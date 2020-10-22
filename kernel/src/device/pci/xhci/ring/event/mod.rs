// SPDX-License-Identifier: GPL-3.0-or-later

use super::Raw;

mod segment_table;

struct EventRing<'a> {
    raw: Raw<'a>,
}
impl<'a> EventRing<'a> {
    fn new(num_trb: usize) -> Self {
        Self {
            raw: Raw::new(num_trb),
        }
    }
}
