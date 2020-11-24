// SPDX-License-Identifier: GPL-3.0-or-later

use super::{super::registers::Registers, raw, CycleBit};
use crate::{
    device::pci::xhci::exchanger::receiver::Receiver,
    mem::allocator::page_box::PageBox,
    multitask::task::{self, Task},
};
use alloc::{rc::Rc, vec::Vec};
use bit_field::BitField;
use core::{
    cell::RefCell,
    convert::{TryFrom, TryInto},
    pin::Pin,
    task::{Context, Poll},
};
use futures_util::{stream::Stream, task::AtomicWaker, StreamExt};
use segment_table::SegmentTable;
use trb::Trb;
use x86_64::{
    structures::paging::{PageSize, Size4KiB},
    PhysAddr,
};

mod segment_table;
pub mod trb;
static WAKER: AtomicWaker = AtomicWaker::new();

pub async fn task_to_check_event_ring() {
    WAKER.wake();
}

pub async fn task(mut ring: Ring, command_completion_receiver: Rc<RefCell<Receiver>>) {
    info!("This is the Event ring task.");
    while let Some(trb) = ring.next().await {
        info!("TRB: {:?}", trb);
        if let Trb::CommandCompletion(trb) = trb {
            info!("Command completion TRB arrived.");
            command_completion_receiver.borrow_mut().receive(trb);
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
const MAX_NUM_OF_TRB_IN_QUEUE: u16 = Size4KiB::SIZE as u16 / Trb::SIZE.as_usize() as u16;

pub struct Ring {
    arrays: Vec<raw::Ring>,
    segment_table: SegmentTable,
    current_cycle_bit: CycleBit,
    dequeue_ptr_trb: usize,
    dequeue_ptr_segment: usize,
    task_collection: Rc<RefCell<task::Collection>>,
    registers: Rc<RefCell<Registers>>,
}
impl<'a> Ring {
    pub fn new(
        registers: Rc<RefCell<Registers>>,
        task_collection: Rc<RefCell<task::Collection>>,
    ) -> Self {
        let max_num_of_erst = registers
            .borrow()
            .capability
            .hcs_params_2
            .read()
            .powered_erst_max();

        Self {
            arrays: Self::new_arrays(max_num_of_erst),
            segment_table: SegmentTable::new(max_num_of_erst.into()),
            current_cycle_bit: CycleBit::new(true),
            dequeue_ptr_trb: 0,
            dequeue_ptr_segment: 0,
            task_collection,
            registers,
        }
    }

    pub fn init(&mut self) {
        self.init_dequeue_ptr();
        self.init_segment_table();
    }

    fn init_dequeue_ptr(&mut self) {
        self.set_dequeue_ptr(self.phys_addr_to_next_trb())
    }

    fn phys_addr_to_segment_table(&self) -> PhysAddr {
        self.segment_table.phys_addr()
    }

    fn init_segment_table(&mut self) {
        self.register_addresses_of_arrays_to_segment_table();
        self.register_segment_table_to_xhci_registers();
    }

    fn register_addresses_of_arrays_to_segment_table(&mut self) {
        for i in 0..self.segment_table.len() {
            self.segment_table[i].set(self.arrays[0].phys_addr(), MAX_NUM_OF_TRB_IN_QUEUE);
        }
    }

    fn register_segment_table_to_xhci_registers(&mut self) {
        let r = &mut self.registers.borrow_mut().runtime;
        r.erst_sz
            .update(|sz| sz.set(self.segment_table.len().try_into().unwrap()));
        r.erst_ba
            .update(|ba| ba.set(self.phys_addr_to_segment_table()));
    }

    fn new_arrays(max_num_of_erst: u16) -> Vec<raw::Ring> {
        let mut arrays = Vec::new();
        for _ in 0_u16..max_num_of_erst {
            arrays.push(raw::Ring::new(MAX_NUM_OF_TRB_IN_QUEUE.into()));
        }

        arrays
    }

    fn dequeue(&mut self) -> Option<Trb> {
        if self.empty() {
            None
        } else {
            let raw = self.arrays[self.dequeue_ptr_segment][self.dequeue_ptr_trb];
            self.increment();

            Some(Trb::try_from(raw).unwrap())
        }
    }

    fn empty(&self) -> bool {
        let raw_trb = self.arrays[self.dequeue_ptr_segment][self.dequeue_ptr_trb];
        raw_trb.cycle_bit() != self.current_cycle_bit
    }

    fn increment(&mut self) {
        self.dequeue_ptr_trb += 1;
        if self.dequeue_ptr_trb >= MAX_NUM_OF_TRB_IN_QUEUE.into() {
            self.dequeue_ptr_trb = 0;
            self.dequeue_ptr_segment += 1;

            if self.dequeue_ptr_segment >= self.num_of_segment_table() {
                self.dequeue_ptr_segment = 0;
                self.current_cycle_bit.toggle();
            }
        }

        self.set_dequeue_ptr(self.phys_addr_to_next_trb())
    }

    fn set_dequeue_ptr(&mut self, addr: PhysAddr) {
        let erd_p = &mut self.registers.borrow_mut().runtime.erd_p;
        erd_p.update(|erd_p| erd_p.set(addr))
    }

    fn phys_addr_to_next_trb(&self) -> PhysAddr {
        self.arrays[self.dequeue_ptr_segment].phys_addr()
            + Trb::SIZE.as_usize() * self.dequeue_ptr_trb
    }

    fn num_of_segment_table(&self) -> usize {
        self.arrays.len()
    }
}
impl<'a> Stream for Ring {
    type Item = Trb;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        WAKER.register(&cx.waker());
        let task_collection = self.task_collection.clone();
        Pin::into_inner(self).dequeue().map_or_else(
            || {
                task_collection
                    .borrow_mut()
                    .add_task_as_woken(Task::new(task_to_check_event_ring()));
                Poll::Pending
            },
            |trb| {
                WAKER.take();
                Poll::Ready(Some(trb))
            },
        )
    }
}

struct Raw {
    rings: Vec<PageBox<[[u32; 4]]>>,
    c: CycleBit,
    deq_p_seg: usize,
    deq_p_trb: usize,
    r: Rc<RefCell<Registers>>,
}
impl Raw {
    fn new(r: Rc<RefCell<Registers>>) -> Self {
        Self {
            rings: Self::new_rings(&r),
            c: CycleBit::new(true),
            deq_p_seg: 0,
            deq_p_trb: 0,
            r,
        }
    }

    fn new_rings(r: &Rc<RefCell<Registers>>) -> Vec<PageBox<[[u32; 4]]>> {
        let mut v = Vec::new();
        for _ in 0..Self::max_num_of_erst(r) {
            v.push(PageBox::new_slice([0; 4], MAX_NUM_OF_TRB_IN_QUEUE.into()));
        }

        v
    }

    fn max_num_of_erst(r: &Rc<RefCell<Registers>>) -> u16 {
        let p2 = r.borrow().capability.hcs_params_2.read();
        p2.powered_erst_max()
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

    fn get_next_trb(&self) -> Result<Trb, Error> {
        let t = self.rings[self.deq_p_seg][self.deq_p_trb];
        t.try_into().or_else(|_| {
            warn!("Unrecognized TRB ID {}", t[3].get_bits(10..=15));
            Err(Error::UnrecognizedTrb)
        })
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
        let p = &mut self.r.borrow_mut().runtime.erd_p;
        p.update(|p| p.set(self.next_trb_addr()));
    }

    fn next_trb_addr(&self) -> PhysAddr {
        self.rings[self.deq_p_seg].phys_addr() + Trb::SIZE.as_usize() * self.deq_p_trb
    }
}

enum Error {
    UnrecognizedTrb,
}
