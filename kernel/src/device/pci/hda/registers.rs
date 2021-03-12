// SPDX-License-Identifier: GPL-3.0-or-later

use crate::mem::{accessor, accessor::Single};
use bit_field::BitField;
use x86_64::PhysAddr;

pub(super) struct Registers {
    pub(super) gctl: Single<GlobalControl>,
}
impl Registers {
    pub(super) unsafe fn new(mmio_base: PhysAddr) -> Self {
        Self {
            gctl: accessor::user(mmio_base),
        }
    }
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub(super) struct GlobalControl(u32);
impl GlobalControl {
    pub(super) fn clear_controller_reset(&mut self) {
        self.0.set_bit(0, false);
    }

    pub(super) fn get_controller_reset(self) -> bool {
        self.0.get_bit(0)
    }
}
