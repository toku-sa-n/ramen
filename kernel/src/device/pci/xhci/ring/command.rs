// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::Raw,
    super::{trb::Trb, CycleBit},
    x86_64::PhysAddr,
};

pub struct Ring {
    raw: Raw,
    enqueue_ptr: usize,
    cycle_bit: CycleBit,
}
impl Ring {
    pub fn new(len: usize) -> Self {
        Self {
            raw: Raw::new(len),
            enqueue_ptr: 0,
            cycle_bit: CycleBit::new(true),
        }
    }

    pub fn phys_addr(&self) -> PhysAddr {
        self.raw.phys_addr()
    }

    fn enqueueable(&self) -> bool {
        let raw = self.raw[self.enqueue_ptr];
        raw.cycle_bit() != self.cycle_bit
    }
}
