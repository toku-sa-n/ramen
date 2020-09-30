// SPDX-License-Identifier: GPL-3.0-or-later

use {crate::accessor::single_object::Accessor, x86_64::PhysAddr};

pub struct RuntimeBaseRegisters<'a> {
    pub erst_sz: Accessor<'a, EventRingSegmentTableSizeRegister>,
    pub erst_ba: Accessor<'a, EventRingSegmentTableBaseAddressRegister>,
    pub erd_p: Accessor<'a, EventRingDequeuePointerRegister>,
}
impl<'a> RuntimeBaseRegisters<'a> {
    pub fn new(mmio_base: PhysAddr, runtime_register_space_offset: usize) -> Self {
        let runtime_base = mmio_base + runtime_register_space_offset;
        let erst_sz = Accessor::new(runtime_base, 0x28);
        let erst_ba = Accessor::new(runtime_base, 0x30);
        let erd_p = Accessor::new(runtime_base, 0x38);

        Self {
            erst_sz,
            erst_ba,
            erd_p,
        }
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct EventRingSegmentTableSizeRegister(u32);
impl EventRingSegmentTableSizeRegister {
    pub fn set(&mut self, val: u32) {
        self.0 = val
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct EventRingSegmentTableBaseAddressRegister(u64);
impl EventRingSegmentTableBaseAddressRegister {
    pub fn set(&mut self, val: PhysAddr) {
        self.0 = val.as_u64()
    }
}

#[repr(transparent)]
pub struct EventRingDequeuePointerRegister(u64);
impl EventRingDequeuePointerRegister {
    pub fn set_address(&mut self, addr: PhysAddr) {
        assert_eq!(addr.as_u64() & 0b1111, 0);
        self.0 = addr.as_u64();
    }
}
