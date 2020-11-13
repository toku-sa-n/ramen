// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::raw,
    super::{super::Registers, trb::Trb, CycleBit},
    alloc::rc::Rc,
    core::cell::RefCell,
    x86_64::PhysAddr,
};

// 4KB / 16 = 256
const SIZE_OF_RING: usize = 256;

pub struct Ring {
    raw: raw::Ring,
    enqueue_ptr: usize,
    cycle_bit: CycleBit,
    registers: Rc<RefCell<Registers>>,
}
impl<'a> Ring {
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

    pub fn send_enable_slot(&mut self) -> Result<PhysAddr, Error> {
        let enable_slot = Trb::new_enable_slot(self.cycle_bit);
        let phys_addr_to_trb = self.enqueue(enable_slot)?;
        self.notify_command_is_sent();
        Ok(phys_addr_to_trb)
    }

    fn notify_command_is_sent(&mut self) {
        let doorbell_array = &mut self.registers.borrow_mut().doorbell_array;
        doorbell_array.update(0, |reg| *reg = 0)
    }

    pub fn init(&mut self) {
        self.register_address_to_xhci_register();
        self.set_initial_command_ring_cycle_state();
    }

    fn register_address_to_xhci_register(&mut self) {
        let crcr = &mut self.registers.borrow_mut().hc_operational.crcr;
        crcr.update(|crcr| crcr.set_ptr(self.phys_addr()));
    }

    fn set_initial_command_ring_cycle_state(&mut self) {
        let crcr = &mut self.registers.borrow_mut().hc_operational.crcr;
        crcr.update(|crcr| crcr.set_ring_cycle_state(true));
    }

    fn enqueue(&mut self, trb: Trb) -> Result<PhysAddr, Error> {
        if !self.enqueueable() {
            return Err(Error::QueueIsFull);
        }

        self.raw[self.enqueue_ptr] = trb.into();

        let phys_addr_to_trb = self.phys_addr_to_enqueue_ptr();

        self.enqueue_ptr += 1;
        if self.enqueue_ptr < self.len() - 1 {
            return Ok(phys_addr_to_trb);
        }

        self.raw[self.enqueue_ptr] = Trb::new_link(self.phys_addr(), self.cycle_bit).into();
        self.enqueue_ptr = 0;
        self.cycle_bit.toggle();

        Ok(phys_addr_to_trb)
    }

    fn phys_addr_to_enqueue_ptr(&self) -> PhysAddr {
        self.phys_addr() + Trb::SIZE.as_usize() * self.enqueue_ptr
    }

    fn enqueueable(&self) -> bool {
        let raw = self.raw[self.enqueue_ptr];
        raw.cycle_bit() != self.cycle_bit
    }

    fn len(&self) -> usize {
        self.raw.len()
    }
}

#[derive(Debug)]
pub enum Error {
    QueueIsFull,
}
