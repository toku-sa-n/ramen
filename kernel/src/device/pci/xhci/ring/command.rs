// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::raw,
    super::{trb::Trb, CycleBit},
    x86_64::PhysAddr,
};

// 4KB / 16 = 256
const SIZE_OF_RING: usize = 256;

pub struct Ring {
    raw: raw::Ring,
    enqueue_ptr: usize,
    cycle_bit: CycleBit,
}
impl Ring {
    pub fn new() -> Self {
        Self {
            raw: raw::Ring::new(SIZE_OF_RING),
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
            info!("Failed to enqueue.");
            return;
        }

        self.raw[self.enqueue_ptr] = trb.into();

        self.enqueue_ptr += 1;
        if self.enqueue_ptr < self.len() - 1 {
            return;
        }

        self.raw[self.enqueue_ptr] = Trb::new_link(self.phys_addr(), self.cycle_bit).into();
        self.enqueue_ptr = 0;
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
