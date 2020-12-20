// SPDX-License-Identifier: GPL-3.0-or-later

use crate::mem::accessor::Accessor;
use bitfield::bitfield;
use os_units::Bytes;
use x86_64::PhysAddr;

pub struct Runtime {
    pub erst_sz: Accessor<EventRingSegmentTableSizeRegister>,
    pub erst_ba: Accessor<EventRingSegmentTableBaseAddressRegister>,
    pub erd_p: Accessor<EventRingDequeuePointerRegister>,
}
impl<'a> Runtime {
    /// SAFETY: This method is unsafe because if `mmio_base` is not a valid address, or
    /// `runtime_register_space_offset` is not a valid value, it can violate memory safety.
    pub unsafe fn new(mmio_base: PhysAddr, runtime_register_space_offset: usize) -> Self {
        let runtime_base = mmio_base + runtime_register_space_offset;
        let erst_sz = Accessor::new(runtime_base, Bytes::new(0x28));
        let erst_ba = Accessor::new(runtime_base, Bytes::new(0x30));
        let erd_p = Accessor::new(runtime_base, Bytes::new(0x38));

        Self {
            erst_sz,
            erst_ba,
            erd_p,
        }
    }
}

bitfield! {
    #[repr(transparent)]
     struct InterruptManagementRegister(u32);

     interrupt_pending,set_interrupt_pending: 0;
     interrupt_enable, set_interrupt_status: 1;
}

bitfield! {
    #[repr(transparent)]
    struct InterruptModerationRegister(u32);

    interrupt_moderation_interval, set_interrupt_interval: 15, 0;
}

#[repr(transparent)]
#[derive(Debug)]
pub struct EventRingSegmentTableSizeRegister(u32);
impl EventRingSegmentTableSizeRegister {
    pub fn set(&mut self, val: u16) {
        self.0 = u32::from(val)
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct EventRingSegmentTableBaseAddressRegister(u64);
impl EventRingSegmentTableBaseAddressRegister {
    pub fn set(&mut self, addr: PhysAddr) {
        let addr = addr.as_u64();
        assert!(addr.trailing_zeros() >= 6);
        self.0 = addr
    }
}

#[repr(transparent)]
pub struct EventRingDequeuePointerRegister(u64);
impl EventRingDequeuePointerRegister {
    pub fn set(&mut self, addr: PhysAddr) {
        assert!(addr.as_u64().trailing_zeros() >= 4);
        self.0 = addr.as_u64();
    }
}
