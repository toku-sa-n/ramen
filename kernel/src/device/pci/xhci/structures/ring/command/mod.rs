// SPDX-License-Identifier: GPL-3.0-or-later

use super::{super::registers::Registers, CycleBit};
use crate::mem::allocator::page_box::PageBox;
use alloc::rc::Rc;
use bit_field::BitField;
use core::cell::RefCell;
use trb::Trb;
use x86_64::PhysAddr;

mod trb;

// 4KB / 16 = 256
const SIZE_OF_RING: usize = 256;

pub struct Ring {
    raw: Raw,
    registers: Rc<RefCell<Registers>>,
}
impl<'a> Ring {
    pub fn new(registers: Rc<RefCell<Registers>>) -> Self {
        Self {
            raw: Raw::new(),
            registers,
        }
    }

    pub fn init(&mut self) {
        Initializer::new(self, self.registers.clone()).init();
    }

    pub fn send_enable_slot(&mut self) -> Result<PhysAddr, Error> {
        let enable_slot = Trb::new_enable_slot();
        let phys_addr_to_trb = self.try_enqueue(enable_slot)?;
        self.notify_command_is_sent();
        Ok(phys_addr_to_trb)
    }

    pub fn send_address_device(
        &mut self,
        addr_to_input_context: PhysAddr,
        slot_id: u8,
    ) -> Result<PhysAddr, Error> {
        let address_device = Trb::new_address_device(addr_to_input_context, slot_id);
        let phys_addr_to_trb = self.try_enqueue(address_device)?;
        self.notify_command_is_sent();
        Ok(phys_addr_to_trb)
    }

    fn try_enqueue(&mut self, trb: Trb) -> Result<PhysAddr, Error> {
        self.raw.try_enqueue(trb)
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
            raw: PageBox::new_slice([0; 4], SIZE_OF_RING),
            enq_p: 0,
            c: CycleBit::new(true),
        }
    }

    fn try_enqueue(&mut self, trb: Trb) -> Result<PhysAddr, Error> {
        if self.enqueueable() {
            Ok(self.enqueue(trb))
        } else {
            Err(Error::QueueIsFull)
        }
    }

    fn enqueueable(&self) -> bool {
        !self.full() && self.has_space_for_link_trb()
    }

    fn full(&self) -> bool {
        !self.writable(self.enq_p)
    }

    fn has_space_for_link_trb(&self) -> bool {
        if self.wrapping_back_happens() {
            self.writable(self.next_enq_p())
        } else {
            true
        }
    }

    fn wrapping_back_happens(&self) -> bool {
        self.next_enq_p() == 0
    }

    fn writable(&self, i: usize) -> bool {
        self.c_bit(i) != self.c
    }

    fn c_bit(&self, i: usize) -> CycleBit {
        let t = self.raw[i];
        CycleBit::new(t[3].get_bit(0))
    }

    fn next_enq_p(&self) -> usize {
        (self.enq_p + 1) % SIZE_OF_RING
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
    registers: Rc<RefCell<Registers>>,
}
impl<'a> Initializer<'a> {
    fn new(ring: &'a Ring, registers: Rc<RefCell<Registers>>) -> Self {
        Self { ring, registers }
    }

    fn init(&mut self) {
        self.register_address_with_xhci();
        self.set_initial_command_ring_cycle_state();
    }

    fn register_address_with_xhci(&mut self) {
        let crcr = &mut self.registers.borrow_mut().operational.crcr;
        crcr.update(|crcr| crcr.set_ptr(self.ring.phys_addr()));
    }

    fn set_initial_command_ring_cycle_state(&mut self) {
        let crcr = &mut self.registers.borrow_mut().operational.crcr;
        crcr.update(|crcr| crcr.set_ring_cycle_state(true));
    }
}

#[derive(Debug)]
pub enum Error {
    QueueIsFull,
}
