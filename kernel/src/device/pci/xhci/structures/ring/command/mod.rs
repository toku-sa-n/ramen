// SPDX-License-Identifier: GPL-3.0-or-later

use super::{super::registers::Registers, CycleBit};
use crate::mem::allocator::page_box::PageBox;
use alloc::rc::Rc;
use core::cell::RefCell;
use trb::Trb;
use x86_64::{
    structures::paging::{PageSize, Size4KiB},
    PhysAddr,
};

pub mod trb;

#[allow(clippy::cast_possible_truncation)]
const NUM_OF_TRBS: usize = Size4KiB::SIZE as usize / Trb::SIZE.as_usize();

pub struct Ring {
    raw: Raw,
    registers: Rc<RefCell<Registers>>,
}
impl Ring {
    pub fn new(registers: Rc<RefCell<Registers>>) -> Self {
        Self {
            raw: Raw::new(),
            registers,
        }
    }

    pub fn init(&mut self) {
        Initializer::new(self, &mut self.registers.borrow_mut()).init();
    }

    pub fn enqueue(&mut self, trb: Trb) -> PhysAddr {
        let a = self.raw.enqueue(trb);
        self.notify_command_is_sent();
        a
    }

    fn phys_addr(&self) -> PhysAddr {
        self.raw.head_addr()
    }

    fn notify_command_is_sent(&mut self) {
        let doorbell_array = &mut self.registers.borrow_mut().doorbell_array;
        doorbell_array.update(0, |reg| *reg = 0)
    }
}

struct Raw {
    raw: PageBox<[[u32; 4]]>,
    enq_p: usize,
    c: CycleBit,
}
impl Raw {
    fn new() -> Self {
        Self {
            raw: PageBox::new_slice([0; 4], NUM_OF_TRBS),
            enq_p: 0,
            c: CycleBit::new(true),
        }
    }

    fn enqueue(&mut self, mut trb: Trb) -> PhysAddr {
        trb.set_c(self.c);
        self.write_trb(trb);
        let trb_a = self.enq_addr();
        self.increment();
        trb_a
    }

    fn write_trb(&mut self, trb: Trb) {
        self.raw[self.enq_p] = trb.into();
    }

    fn increment(&mut self) {
        self.enq_p += 1;
        if !self.enq_p_within_ring() {
            self.enq_link();
            self.move_enq_p_to_the_beginning();
        }
    }

    fn enq_p_within_ring(&self) -> bool {
        self.enq_p < self.len() - 1
    }

    fn enq_link(&mut self) {
        // Don't call `enqueue`. It will return an `Err` value as there is no space for link TRB.
        let mut t = Trb::new_link(self.head_addr());
        t.set_c(self.c);
        self.raw[self.enq_p] = t.into();
    }

    fn move_enq_p_to_the_beginning(&mut self) {
        self.enq_p = 0;
        self.c.toggle();
    }

    fn enq_addr(&self) -> PhysAddr {
        self.head_addr() + Trb::SIZE.as_usize() * self.enq_p
    }

    fn head_addr(&self) -> PhysAddr {
        self.raw.phys_addr()
    }

    fn len(&self) -> usize {
        self.raw.len()
    }
}

struct Initializer<'a> {
    ring: &'a Ring,
    registers: &'a mut Registers,
}
impl<'a> Initializer<'a> {
    fn new(ring: &'a Ring, registers: &'a mut Registers) -> Self {
        Self { ring, registers }
    }

    fn init(&mut self) {
        self.register_address_with_xhci();
        self.set_initial_command_ring_cycle_state();
    }

    fn register_address_with_xhci(&mut self) {
        let ring_addr = self.ring.phys_addr();
        let crcr = &mut self.registers.operational.crcr;
        crcr.update(|crcr| crcr.set_ptr(ring_addr));
    }

    fn set_initial_command_ring_cycle_state(&mut self) {
        let crcr = &mut self.registers.operational.crcr;
        crcr.update(|crcr| crcr.set_ring_cycle_state(true));
    }
}
