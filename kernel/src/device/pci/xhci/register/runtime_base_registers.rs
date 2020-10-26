// SPDX-License-Identifier: GPL-3.0-or-later

use {crate::mem::accessor::Accessor, bitfield::bitfield, os_units::Bytes, x86_64::PhysAddr};

pub struct RuntimeBaseRegisters {
    i_man: Accessor<InterruptManagementRegister>,
    i_mod: Accessor<InterruptModerationRegister>,
    erst_sz: Accessor<EventRingSegmentTableSizeRegister>,
    erst_ba: Accessor<EventRingSegmentTableBaseAddressRegister>,
    erd_p: Accessor<EventRingDequeuePointerRegister>,
}
impl<'a> RuntimeBaseRegisters {
    pub fn new(mmio_base: PhysAddr, runtime_register_space_offset: usize) -> Self {
        let runtime_base = mmio_base + runtime_register_space_offset;
        let i_man = Accessor::new(runtime_base, Bytes::new(0x20));
        let i_mod = Accessor::new(runtime_base, Bytes::new(0x24));
        let erst_sz = Accessor::new(runtime_base, Bytes::new(0x28));
        let erst_ba = Accessor::new(runtime_base, Bytes::new(0x30));
        let erd_p = Accessor::new(runtime_base, Bytes::new(0x38));

        Self {
            i_man,
            i_mod,
            erst_sz,
            erst_ba,
            erd_p,
        }
    }

    pub fn set_event_ring_segment_table_size(&mut self, size: u16) {
        self.erst_sz.set(size)
    }

    pub fn set_event_ring_segment_table_addr(&mut self, addr: PhysAddr) {
        self.erst_ba.set(addr)
    }

    pub fn set_event_ring_dequeue_ptr(&mut self, ptr: PhysAddr) {
        self.erd_p.set_address(ptr)
    }

    pub fn enable_interrupt(&mut self) {
        self.i_man.set_interrupt_pending(true);
        self.i_man.set_interrupt_status(true);

        self.i_mod.set_interrupt_interval(4000);
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
struct EventRingSegmentTableSizeRegister(u32);
impl EventRingSegmentTableSizeRegister {
    fn set(&mut self, val: u16) {
        self.0 = u32::from(val)
    }
}

#[repr(transparent)]
#[derive(Debug)]
struct EventRingSegmentTableBaseAddressRegister(u64);
impl EventRingSegmentTableBaseAddressRegister {
    fn set(&mut self, val: PhysAddr) {
        assert!(val.as_u64().trailing_zeros() >= 6);
        self.0 = val.as_u64()
    }
}

#[repr(transparent)]
struct EventRingDequeuePointerRegister(u64);
impl EventRingDequeuePointerRegister {
    fn set_address(&mut self, addr: PhysAddr) {
        assert!(addr.as_u64().trailing_zeros() >= 4);
        self.0 = addr.as_u64();
    }
}
