// SPDX-License-Identifier: GPL-3.0-or-later

use super::CycleBit;
use crate::{device::pci::xhci, mem::allocator::page_box::PageBox};
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
}
impl Ring {
    pub fn new() -> Self {
        Self { raw: Raw::new() }
    }

    pub fn init(&mut self) {
        Initializer::new(self).init();
    }

    pub fn enqueue(&mut self, trb: Trb) -> PhysAddr {
        let a = self.raw.enqueue(trb);
        Self::notify_command_is_sent();
        a
    }

    fn phys_addr(&self) -> PhysAddr {
        self.raw.head_addr()
    }

    fn notify_command_is_sent() {
        xhci::handle_registers(|r| {
            let d = &mut r.doorbell_array;
            d.update(0, |reg| *reg = 0)
        })
    }
}
impl Default for Ring {
    fn default() -> Self {
        Self::new()
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
            raw: PageBox::user_slice([0; 4], NUM_OF_TRBS),
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
        // TODO: Write four 32-bit values. This way of writing is described in the spec, although
        // I cannot find which section has the description.
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
}
impl<'a> Initializer<'a> {
    fn new(ring: &'a Ring) -> Self {
        Self { ring }
    }

    fn init(&mut self) {
        xhci::handle_registers(|r| {
            let a = self.ring.phys_addr();
            let c = &mut r.operational.crcr;

            // Do not split this closure to avoid read-modify-write bug. Reading fields may return
            // 0, this will cause writing 0 to fields.
            c.update(|c| {
                c.set_ptr(a);
                c.set_ring_cycle_state(true);
                info!("CRCR: {:X?}", c);
            });
        })
    }
}
