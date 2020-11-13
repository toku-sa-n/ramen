// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{super::Registers, raw, CycleBit},
    alloc::rc::Rc,
    core::cell::RefCell,
};

const SIZE_OF_RING: usize = 256;

pub struct Ring {
    raw: raw::Ring,
    enqueue_ptr: usize,
    cycle_bit: CycleBit,
    registers: Rc<RefCell<Registers>>,
}
impl Ring {
    pub fn new(registers: Rc<RefCell<Registers>>) -> Self {
        Self {
            raw: raw::Ring::new(SIZE_OF_RING),
            enqueue_ptr: 0,
            cycle_bit: CycleBit::new(true),
            registers,
        }
    }
}
