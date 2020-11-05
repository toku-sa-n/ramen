// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::raw,
    super::{super::Registers, trb::Trb, CycleBit},
    spinning_top::Spinlock,
    x86_64::PhysAddr,
};

// 4KB / 16 = 256
const SIZE_OF_RING: usize = 256;

pub struct Ring<'a> {
    raw: raw::Ring,
    enqueue_ptr: usize,
    cycle_bit: CycleBit,
    registers: &'a Spinlock<Registers>,
}
impl<'a> Ring<'a> {
    pub fn new(registers: &'a Spinlock<Registers>) -> Self {
        let mut command_ring = Self {
            raw: raw::Ring::new(SIZE_OF_RING),
            enqueue_ptr: 0,
            cycle_bit: CycleBit::new(true),
            registers,
        };
        command_ring
    }

    pub fn phys_addr(&self) -> PhysAddr {
        self.raw.phys_addr()
    }

    pub fn send_noop(&mut self) {
        let noop = Trb::new_noop(self.cycle_bit);
        self.enqueue(noop);
        self.notify_command_is_sent();
    }

    fn notify_command_is_sent(&mut self) {
        self.registers.lock().doorbell_array[0] = 0;
    }

    pub fn init(&mut self) {
        self.register_address_to_xhci_register();
    }

    fn register_address_to_xhci_register(&mut self) {
        self.registers
            .lock()
            .hc_operational
            .crcr
            .set_ptr(self.phys_addr());
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
