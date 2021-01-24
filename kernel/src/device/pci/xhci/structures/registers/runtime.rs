// SPDX-License-Identifier: GPL-3.0-or-later

use crate::mem::accessor::Single;
use x86_64::PhysAddr;
use xhci::registers::runtime::{
    EventRingDequeuePointerRegister, EventRingSegmentTableBaseAddressRegister,
    EventRingSegmentTableSizeRegister,
};

pub struct Runtime {
    pub erst_sz: Single<EventRingSegmentTableSizeRegister>,
    pub erst_ba: Single<EventRingSegmentTableBaseAddressRegister>,
    pub erd_p: Single<EventRingDequeuePointerRegister>,
}
impl<'a> Runtime {
    /// SAFETY: This method is unsafe because if `mmio_base` is not a valid address, or
    /// `runtime_register_space_offset` is not a valid value, it can violate memory safety.
    pub unsafe fn new(mmio_base: PhysAddr, runtime_register_space_offset: usize) -> Self {
        let runtime_base = mmio_base + runtime_register_space_offset;
        let erst_sz =
            crate::mem::accessor::user(runtime_base + 0x28_u64).expect("Address is not aligned.");
        let erst_ba =
            crate::mem::accessor::user(runtime_base + 0x30_u64).expect("Address is not aligned.");
        let erd_p =
            crate::mem::accessor::user(runtime_base + 0x38_u64).expect("Address is not aligned.");

        Self {
            erst_sz,
            erst_ba,
            erd_p,
        }
    }
}
