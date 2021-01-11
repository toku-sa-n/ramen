// SPDX-License-Identifier: GPL-3.0-or-later

use super::CycleBit;
use crate::{
    device::pci::xhci::{self, exchanger::receiver::Receiver},
    mem::allocator::page_box::PageBox,
};
use alloc::{sync::Arc, vec::Vec};
use bit_field::BitField;
use core::{
    convert::TryInto,
    pin::Pin,
    task::{Context, Poll},
};
use futures_util::{stream::Stream, StreamExt};
use segment_table::SegmentTable;
use spinning_top::Spinlock;
use trb::Trb;
use x86_64::{
    structures::paging::{PageSize, Size4KiB},
    PhysAddr,
};
use xhci::port;

mod segment_table;
pub mod trb;

pub async fn task(mut ring: Ring, command_completion_receiver: Arc<Spinlock<Receiver>>) {
    debug!("This is the Event ring task.");
    while let Some(trb) = ring.next().await {
        info!("TRB: {:?}", trb);
        if let Trb::Completion(trb) = trb {
            debug!("Command completion TRB arrived.");
            command_completion_receiver.lock().receive(trb);
        } else if let Trb::PortStatusChange(t) = trb {
            let _ = port::try_spawn(t.port());
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
const MAX_NUM_OF_TRB_IN_QUEUE: u16 = Size4KiB::SIZE as u16 / Trb::SIZE.as_usize() as u16;

pub struct Ring {
    segment_table: SegmentTable,
    raw: Raw,
}
impl<'a> Ring {
    pub fn new() -> Self {
        let max_num_of_erst = xhci::handle_registers(|r| {
            let p2 = r.capability.hcs_params_2.read();
            p2.powered_erst_max()
        });

        Self {
            segment_table: SegmentTable::new(max_num_of_erst.into()),
            raw: Raw::new(),
        }
    }

    pub fn init(&mut self) {
        self.init_dequeue_ptr();
        self.init_tbl();
    }

    fn init_dequeue_ptr(&mut self) {
        self.raw.update_deq_p_with_xhci()
    }

    fn phys_addr_to_segment_table(&self) -> PhysAddr {
        self.segment_table.phys_addr()
    }

    fn init_tbl(&mut self) {
        SegTblInitializer::new(self).init();
    }

    fn try_dequeue(&mut self) -> Option<Trb> {
        self.raw.try_dequeue()
    }

    fn ring_addrs(&self) -> Vec<PhysAddr> {
        self.raw.head_addrs()
    }

    fn iter_tbl_entries_mut(&mut self) -> impl Iterator<Item = &mut segment_table::Entry> {
        self.segment_table.iter_mut()
    }
}
impl<'a> Stream for Ring {
    type Item = Trb;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::into_inner(self)
            .try_dequeue()
            .map_or_else(|| Poll::Pending, |trb| Poll::Ready(Some(trb)))
    }
}

struct Raw {
    rings: Vec<PageBox<[[u32; 4]]>>,
    c: CycleBit,
    deq_p_seg: usize,
    deq_p_trb: usize,
}
impl Raw {
    fn new() -> Self {
        let rings = Self::new_rings();
        Self {
            rings,
            c: CycleBit::new(true),
            deq_p_seg: 0,
            deq_p_trb: 0,
        }
    }

    fn new_rings() -> Vec<PageBox<[[u32; 4]]>> {
        let mut v = Vec::new();
        for _ in 0..Self::max_num_of_erst() {
            v.push(PageBox::user_slice([0; 4], MAX_NUM_OF_TRB_IN_QUEUE.into()));
        }

        v
    }

    fn max_num_of_erst() -> u16 {
        xhci::handle_registers(|r| {
            let p = r.capability.hcs_params_2.read();
            p.powered_erst_max()
        })
    }

    fn try_dequeue(&mut self) -> Option<Trb> {
        if self.empty() {
            None
        } else {
            self.dequeue()
        }
    }

    fn empty(&self) -> bool {
        self.c_bit_of_next_trb() != self.c
    }

    fn c_bit_of_next_trb(&self) -> CycleBit {
        let t = self.rings[self.deq_p_seg][self.deq_p_trb];
        CycleBit::new(t[3].get_bit(0))
    }

    fn dequeue(&mut self) -> Option<Trb> {
        let t = self.get_next_trb().ok();
        self.increment();
        t
    }

    fn get_next_trb(&self) -> Result<Trb, trb::Error> {
        let r = self.rings[self.deq_p_seg][self.deq_p_trb];
        let t = r.try_into();
        if t.is_err() {
            warn!("Unrecognized ID: {}", r[3].get_bits(10..=15));
        }
        t
    }

    fn increment(&mut self) {
        self.deq_p_trb += 1;
        if self.deq_p_trb >= MAX_NUM_OF_TRB_IN_QUEUE.into() {
            self.deq_p_trb = 0;
            self.deq_p_seg += 1;

            if self.deq_p_seg >= self.num_of_erst() {
                self.deq_p_seg = 0;
                self.c.toggle();
            }
        }
    }

    fn num_of_erst(&self) -> usize {
        self.rings.len()
    }

    fn update_deq_p_with_xhci(&self) {
        xhci::handle_registers(|r| {
            let p = &mut r.runtime.erd_p;
            p.update(|p| p.set(self.next_trb_addr()));
        });
    }

    fn next_trb_addr(&self) -> PhysAddr {
        self.rings[self.deq_p_seg].phys_addr() + Trb::SIZE.as_usize() * self.deq_p_trb
    }

    fn head_addrs(&self) -> Vec<PhysAddr> {
        self.rings.iter().map(PageBox::phys_addr).collect()
    }
}

struct SegTblInitializer<'a> {
    ring: &'a mut Ring,
}
impl<'a> SegTblInitializer<'a> {
    fn new(ring: &'a mut Ring) -> Self {
        Self { ring }
    }

    fn init(&mut self) {
        self.write_addrs();
        self.register_tbl_sz();
        self.enable_event_ring();
    }

    fn write_addrs(&mut self) {
        let addrs = self.ring.ring_addrs();
        for (entry, addr) in self.ring.iter_tbl_entries_mut().zip(addrs) {
            entry.set(addr, MAX_NUM_OF_TRB_IN_QUEUE);
        }
    }

    fn register_tbl_sz(&mut self) {
        xhci::handle_registers(|r| {
            let l = self.tbl_len();
            let s = &mut r.runtime.erst_sz;

            s.update(|s| s.set(l.try_into().unwrap()));
        })
    }

    fn enable_event_ring(&mut self) {
        xhci::handle_registers(|r| {
            let a = self.tbl_addr();
            let b = &mut r.runtime.erst_ba;

            b.update(|b| b.set(a));
        });
    }

    fn tbl_addr(&self) -> PhysAddr {
        self.ring.phys_addr_to_segment_table()
    }

    fn tbl_len(&self) -> usize {
        self.ring.segment_table.len()
    }
}
