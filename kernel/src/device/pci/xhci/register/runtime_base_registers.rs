// SPDX-License-Identifier: GPL-3.0-or-later

use {crate::accessor::single_object::Accessor, bitfield::bitfield, x86_64::PhysAddr};

pub struct RuntimeBaseRegisters<'a> {
    i_man: Accessor<'a, InterruptManagementRegister>,
    erst_sz: Accessor<'a, EventRingSegmentTableSizeRegister>,
    erst_ba: Accessor<'a, EventRingSegmentTableBaseAddressRegister>,
    erd_p: Accessor<'a, EventRingDequeuePointerRegister>,
}
impl<'a> RuntimeBaseRegisters<'a> {
    pub fn new(mmio_base: PhysAddr, runtime_register_space_offset: usize) -> Self {
        let runtime_base = mmio_base + runtime_register_space_offset;
        let i_man = Accessor::new(runtime_base, 0x20);
        let erst_sz = Accessor::new(runtime_base, 0x28);
        let erst_ba = Accessor::new(runtime_base, 0x30);
        let erd_p = Accessor::new(runtime_base, 0x38);

        Self {
            i_man,
            erst_sz,
            erst_ba,
            erd_p,
        }
    }

    pub fn set_event_ring_size(&mut self, size: u16) {
        self.erst_sz.set(size)
    }

    pub fn set_event_ring_addr(&mut self, addr: PhysAddr) {
        self.erst_ba.set(addr)
    }

    pub fn set_event_ring_dequeue_ptr(&mut self, ptr: PhysAddr) {
        self.erd_p.set_address(ptr)
    }

    pub fn enable_interrupt(&mut self) {
        self.i_man.set_interrupt_status(true)
    }
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
        assert_eq!(val.as_u64() & 0x3f, 0);
        self.0 = val.as_u64()
    }
}

#[repr(transparent)]
struct EventRingDequeuePointerRegister(u64);
impl EventRingDequeuePointerRegister {
    fn set_address(&mut self, addr: PhysAddr) {
        assert_eq!(addr.as_u64() & 0b1111, 0);
        self.0 = addr.as_u64();
    }
}

bitfield! {
    #[repr(transparent)]
     struct InterruptManagementRegister(u32);

     interrupt_enable, set_interrupt_status: 1;
}
