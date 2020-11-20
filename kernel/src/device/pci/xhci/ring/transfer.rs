// SPDX-License-Identifier: GPL-3.0-or-later

use super::{super::Registers, raw, trb::Trb, CycleBit};
use alloc::rc::Rc;
use core::cell::RefCell;
use x86_64::PhysAddr;

const SIZE_OF_RING: usize = 256;

pub struct Ring {
    raw: raw::Ring,
    enqueue_ptr: usize,
    cycle_bit: CycleBit,
    registers: Rc<RefCell<Registers>>,
}
impl Ring {
    pub fn new(registers: Rc<RefCell<Registers>>) -> Self {
        Self {
            raw: raw::Ring::new(SIZE_OF_RING),
            enqueue_ptr: 0,
            cycle_bit: CycleBit::new(true),
            registers,
        }
    }

    pub fn phys_addr(&self) -> PhysAddr {
        self.raw.phys_addr()
    }

    fn enqueue(&mut self, trb: Trb) -> Result<PhysAddr, Error> {
        if self.full() {
            return Err(Error::QueueIsFull);
        }

        self.raw[self.enqueue_ptr] = trb.into();

        let addr_to_trb = self.addr_to_enqueue_ptr();
        self.enqueue_ptr += 1;
        if self.enqueue_ptr < self.len() {
            return Ok(addr_to_trb);
        }

        self.raw[self.enqueue_ptr] = Trb::new_link(self.phys_addr(), self.cycle_bit).into();

        self.enqueue_ptr = 0;
        self.cycle_bit.toggle();

        Ok(addr_to_trb)
    }

    fn full(&self) -> bool {
        let raw = self.raw[self.enqueue_ptr];
        raw.cycle_bit() == self.cycle_bit
    }

    fn addr_to_enqueue_ptr(&self) -> PhysAddr {
        self.phys_addr() + Trb::SIZE.as_usize() * self.enqueue_ptr
    }

    fn len(&self) -> usize {
        self.raw.len()
    }
}

#[derive(Debug)]
enum Error {
    QueueIsFull,
}
