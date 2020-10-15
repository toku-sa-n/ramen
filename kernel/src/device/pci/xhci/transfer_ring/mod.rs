// SPDX-License-Identifier: GPL-3.0-or-later

use {
    core::{
        marker::PhantomData,
        ops::{Index, IndexMut},
    },
    x86_64::VirtAddr,
};

// 4KB / size_of(TRB) = 256.
const NUM_OF_TRB_IN_QUEUE: usize = 256;

static EVENT_RING: Ring<Event> = Ring::<Event>::new();
static COMMAND_RING: Ring<Command> = Ring::<Command>::new();

#[repr(align(32))]
struct Ring<T: TrbType>([Trb<T>; NUM_OF_TRB_IN_QUEUE]);
impl Ring<Event> {
    const fn new() -> Self {
        const TRB: Trb<Event> = Trb::new();
        Self([TRB; NUM_OF_TRB_IN_QUEUE])
    }
}
impl Ring<Command> {
    const fn new() -> Self {
        const TRB: Trb<Command> = Trb::new();
        Self([TRB; NUM_OF_TRB_IN_QUEUE])
    }
}
impl<T: TrbType> Index<usize> for Ring<T> {
    type Output = Trb<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

pub struct RingQueue<T: TrbType + 'static> {
    queue: &'static Ring<T>,
    dequeue_index: usize,
    cycle_bit: CycleBit,
}

impl<T: TrbType + 'static> RingQueue<T> {
    pub fn addr(&self) -> VirtAddr {
        VirtAddr::new(self.queue as *const _ as _)
    }
}

impl RingQueue<Event> {
    pub fn new() -> Self {
        Self {
            queue: &EVENT_RING,
            dequeue_index: 0,
            cycle_bit: CycleBit::new(1),
        }
    }
    pub fn dequeue(&mut self) -> Option<Trb<Event>> {
        if self.queue[self.dequeue_index].valid(self.cycle_bit) {
            let element = self.queue[self.dequeue_index];
            self.increment_dequeue_index();
            Some(element)
        } else {
            None
        }
    }

    fn increment_dequeue_index(&mut self) {
        self.dequeue_index += 1;
        if self.dequeue_index >= NUM_OF_TRB_IN_QUEUE {
            self.dequeue_index = 0;
            self.cycle_bit.toggle();
        }
    }
}

impl RingQueue<Command> {
    pub fn new() -> Self {
        Self {
            queue: &COMMAND_RING,
            dequeue_index: 0,
            cycle_bit: CycleBit::new(1),
        }
    }
}

pub trait TrbType {
    const SIZE: usize = 16;
}

#[derive(Copy, Clone, Debug)]
pub struct Command;
impl TrbType for Command {}

#[derive(Copy, Clone, Debug)]
pub struct Event;
impl TrbType for Event {}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Trb<T: TrbType> {
    trb: [u32; 4],
    _marker: PhantomData<T>,
}
impl<T: TrbType> Trb<T> {
    const fn new() -> Self {
        Self {
            trb: [0; 4],
            _marker: PhantomData,
        }
    }

    pub fn ty(&self) -> u32 {
        (self.trb[3] >> 10) & 0x3f
    }

    fn valid(&self, cycle_bit: CycleBit) -> bool {
        self.trb[3] & 1 == cycle_bit.0
    }
}

#[derive(Copy, Clone)]
struct CycleBit(u32);
impl CycleBit {
    fn new(bit: u32) -> Self {
        assert!(bit == 0 || bit == 1);
        Self(bit)
    }

    fn toggle(&mut self) {
        self.0 ^= 1;
    }
}
