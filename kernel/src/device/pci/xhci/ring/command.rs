// SPDX-License-Identifier: GPL-3.0-or-later

use {super::Raw, x86_64::PhysAddr};

pub struct Ring {
    raw: Raw,
    enqueue_ptr: usize,
    len: usize,
}
impl Ring {
    pub fn new(len: usize) -> Self {
        Self {
            raw: Raw::new(len),
            enqueue_ptr: 0,
            len,
        }
    }

    pub fn phys_addr(&self) -> PhysAddr {
        self.raw.phys_addr()
    }
}
