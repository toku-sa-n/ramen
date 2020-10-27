// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::raw,
    super::{trb::Trb, CycleBit},
    x86_64::PhysAddr,
};

pub struct Ring {
    raw: raw::Ring,
    enqueue_ptr: usize,
    cycle_bit: CycleBit,
}
impl Ring {
    pub fn new(len: usize) -> Self {
        Self {
            raw: raw::Ring::new(len),
            enqueue_ptr: 0,
            cycle_bit: CycleBit::new(true),
        }
    }

    pub fn phys_addr(&self) -> PhysAddr {
        self.raw.phys_addr()
    }

    pub fn send_noop(&mut self) {
        let noop = Trb::new_noop(self.cycle_bit);
        self.enqueue(noop);
    }

    fn enqueue(&mut self, trb: Trb) {
        if !self.enqueueable() {
            return;
        }

        self.raw[self.enqueue_ptr] = trb.into();

        self.enqueue_ptr += 1;
        if self.enqueue_ptr < self.len() {
            return;
        }

        self.enqueue_ptr %= self.len();
        self.cycle_bit.toggle();
    }

    fn enqueueable(&self) -> bool {
        let raw = self.raw[self.enqueue_ptr];
        raw.cycle_bit() != self.cycle_bit
    }

    fn len(&self) -> usize {
        self.raw.len()
    }
}
