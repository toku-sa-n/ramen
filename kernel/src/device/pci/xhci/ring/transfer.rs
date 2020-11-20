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

    fn try_enqueue(&mut self, trb: Trb) -> Result<PhysAddr, Error> {
        if self.full() {
            Err(Error::QueueIsFull)
        } else {
            Ok(self.enqueue(trb))
        }
    }

    fn full(&self) -> bool {
        let raw = self.raw[self.enqueue_ptr];
        raw.cycle_bit() == self.cycle_bit
    }

    fn enqueue(&mut self, trb: Trb) -> PhysAddr {
        self.write_trb_on_memory(trb);
        let addr_to_trb = self.addr_to_enqueue_ptr();
        self.increment_enqueue_ptr();

        addr_to_trb
    }

    fn write_trb_on_memory(&mut self, trb: Trb) {
        self.raw[self.enqueue_ptr] = trb.into();
    }

    fn addr_to_enqueue_ptr(&self) -> PhysAddr {
        self.phys_addr() + Trb::SIZE.as_usize() * self.enqueue_ptr
    }

    fn increment_enqueue_ptr(&mut self) {
        self.enqueue_ptr += 1;
        if self.enqueue_ptr < self.len() - 1 {
            return;
        }

        self.append_link_trb();
        self.move_enqueue_ptr_to_the_beginning();
    }

    fn len(&self) -> usize {
        self.raw.len()
    }

    fn append_link_trb(&mut self) {
        self.raw[self.enqueue_ptr] = Trb::new_link(self.phys_addr(), self.cycle_bit).into();
    }

    fn move_enqueue_ptr_to_the_beginning(&mut self) {
        self.enqueue_ptr = 0;
        self.cycle_bit.toggle();
    }
}

#[derive(Debug)]
enum Error {
    QueueIsFull,
}
