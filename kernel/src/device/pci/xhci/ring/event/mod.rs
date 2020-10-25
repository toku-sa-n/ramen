// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{trb::Trb, CycleBit, Raw},
    crate::device::pci::xhci,
    core::{
        convert::TryFrom,
        pin::Pin,
        task::{Context, Poll},
    },
    futures_util::stream::Stream,
};

mod segment_table;

struct EventRing {
    raw: Raw,
    current_cycle_bit: CycleBit,
    dequeue_ptr: usize,
}
impl<'a> EventRing {
    fn new(len: usize) -> Self {
        Self {
            raw: Raw::new(len),
            current_cycle_bit: CycleBit::new(true),
            dequeue_ptr: 0,
        }
    }

    fn dequeue(&mut self) -> Option<Trb> {
        if self.empty() {
            None
        } else {
            let raw = self.raw[self.dequeue_ptr];
            self.increment();

            Some(Trb::try_from(raw).unwrap())
        }
    }

    fn empty(&self) -> bool {
        let raw_trb = self.raw[self.dequeue_ptr];
        if Trb::try_from(raw_trb).is_ok() {
            CycleBit::from(raw_trb) != self.current_cycle_bit
        } else {
            true
        }
    }

    fn increment(&mut self) {
        self.dequeue_ptr += 1;
        if self.dequeue_ptr >= self.len() {
            self.dequeue_ptr %= self.len();
            self.current_cycle_bit.toggle();
        }
    }

    fn len(&self) -> usize {
        self.raw.len()
    }
}
impl<'a> Stream for EventRing {
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
