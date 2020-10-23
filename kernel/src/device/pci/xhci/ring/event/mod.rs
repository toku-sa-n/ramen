// SPDX-License-Identifier: GPL-3.0-or-later

use super::{CycleBit, Raw};

mod segment_table;

struct EventRing<'a> {
    raw: Raw<'a>,
    current_cycle_bit: CycleBit,
    dequeue_ptr: usize,
}
impl<'a> EventRing<'a> {
    fn new(num_trb: usize) -> Self {
        Self {
            raw: Raw::new(num_trb),
            current_cycle_bit: CycleBit::new(true),
            dequeue_ptr: 0,
        }
    }
}
