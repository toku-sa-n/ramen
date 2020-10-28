// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{super::register::hc_capability_registers::MaxNumOfErst, raw, trb::Trb, CycleBit},
    crate::device::pci::xhci,
    alloc::vec::Vec,
    core::{
        convert::TryFrom,
        pin::Pin,
        task::{Context, Poll},
    },
    futures_util::stream::Stream,
    segment_table::SegmentTable,
    x86_64::PhysAddr,
};

mod segment_table;

pub struct Ring {
    arrays: Vec<raw::Ring>,
    segment_table: SegmentTable,
    current_cycle_bit: CycleBit,
    dequeue_ptr_trb: usize,
    dequeue_ptr_segment: usize,
}
impl<'a> Ring {
    const MAX_NUM_OF_TRB_IN_QUEUE: u16 = 4096;

    pub fn new(max_num_of_erst: MaxNumOfErst) -> Self {
        let mut ring = Self {
            arrays: Self::new_arrays(max_num_of_erst),
            segment_table: SegmentTable::new(u16::from(max_num_of_erst).into()),
            current_cycle_bit: CycleBit::new(true),
            dequeue_ptr_trb: 0,
            dequeue_ptr_segment: 0,
        };
        ring.init_segment_table();
        ring
    }

    pub fn phys_addr_to_array_beginning(&self) -> PhysAddr {
        self.arrays[0].phys_addr()
    }

    pub fn phys_addr_to_segment_table(&self) -> PhysAddr {
        self.segment_table.phys_addr()
    }

    fn init_segment_table(&mut self) {
        for i in 0..self.segment_table.len() {
            self.segment_table[i].set(self.arrays[0].phys_addr(), Self::MAX_NUM_OF_TRB_IN_QUEUE);
        }
    }

    fn new_arrays(max_num_of_erst: MaxNumOfErst) -> Vec<raw::Ring> {
        let mut arrays = Vec::new();
        for _ in 0..max_num_of_erst.into() {
            arrays.push(raw::Ring::new(Self::MAX_NUM_OF_TRB_IN_QUEUE.into()));
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
        CycleBit::from(raw_trb) != self.current_cycle_bit
    }

    fn increment(&mut self) {
        self.dequeue_ptr_trb += 1;
        if self.dequeue_ptr_trb >= Self::MAX_NUM_OF_TRB_IN_QUEUE.into() {
            self.dequeue_ptr_trb = 0;
            self.dequeue_ptr_segment += 1;

            if self.dequeue_ptr_segment >= self.num_of_segment_table() {
                self.dequeue_ptr_segment = 0;
                self.current_cycle_bit.toggle();
            }
        }
    }

    fn num_of_segment_table(&self) -> usize {
        self.arrays.len()
    }
}
impl<'a> Stream for Ring {
    type Item = Trb;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        xhci::WAKER.register(&cx.waker());
        match Pin::into_inner(self).dequeue() {
            Some(trb) => {
                xhci::WAKER.take();
                Poll::Ready(Some(trb))
            }
            None => Poll::Pending,
        }
    }
}
