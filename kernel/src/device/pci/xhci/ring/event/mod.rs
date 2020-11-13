// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{
        super::{command_runner::CommandCompletionReceiver, register::Registers},
        raw,
        trb::Trb,
        CycleBit,
    },
    crate::multitask::task::{self, Task},
    alloc::{rc::Rc, vec::Vec},
    core::{
        cell::RefCell,
        convert::{TryFrom, TryInto},
        pin::Pin,
        task::{Context, Poll},
    },
    futures_util::{stream::Stream, task::AtomicWaker, StreamExt},
    segment_table::SegmentTable,
    x86_64::PhysAddr,
};

mod segment_table;
static WAKER: AtomicWaker = AtomicWaker::new();

pub async fn task_to_check_event_ring() {
    WAKER.wake();
}

pub async fn task(
    mut ring: Ring,
    command_completion_receiver: Rc<RefCell<CommandCompletionReceiver>>,
) {
    info!("This is the Event ring task.");
    while let Some(trb) = ring.next().await {
        info!("TRB: {:?}", trb);
        if let Trb::CommandComplete(trb) = trb {
            info!("Command completion TRB arrived.");
            command_completion_receiver.borrow_mut().receive(trb);
        }
    }
}

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
    const MAX_NUM_OF_TRB_IN_QUEUE: u16 = 4096;

    pub fn new(
        registers: Rc<RefCell<Registers>>,
        task_collection: Rc<RefCell<task::Collection>>,
    ) -> Self {
        let max_num_of_erst = registers
            .borrow()
            .hc_capability
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
            self.segment_table[i].set(self.arrays[0].phys_addr(), Self::MAX_NUM_OF_TRB_IN_QUEUE);
        }
    }

    fn register_segment_table_to_xhci_registers(&mut self) {
        let runtime_registers = &mut self.registers.borrow_mut().runtime_base_registers;
        runtime_registers
            .erst_sz
            .update(|sz| sz.set(self.segment_table.len().try_into().unwrap()));
        runtime_registers
            .erst_ba
            .update(|ba| ba.set(self.phys_addr_to_segment_table()));
    }

    fn new_arrays(max_num_of_erst: u16) -> Vec<raw::Ring> {
        let mut arrays = Vec::new();
        for _ in 0_u16..max_num_of_erst {
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
        raw_trb.cycle_bit() != self.current_cycle_bit
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

        self.set_dequeue_ptr(self.phys_addr_to_next_trb())
    }

    fn set_dequeue_ptr(&mut self, addr: PhysAddr) {
        let erd_p = &mut self.registers.borrow_mut().runtime_base_registers.erd_p;
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
