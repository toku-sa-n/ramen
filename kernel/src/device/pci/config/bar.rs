// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{Bus, ConfigAddress, Device, Function, Offset},
    x86_64::PhysAddr,
};

#[derive(Debug)]
pub struct Bar(u64);

impl Bar {
    pub(super) fn fetch(bus: Bus, device: Device) -> Self {
        let config_addr_low = ConfigAddress::new(bus, device, Function::zero(), Offset::new(0x10));
        let low_bar = unsafe { config_addr_low.read() };

        let config_addr_high = ConfigAddress::new(bus, device, Function::zero(), Offset::new(0x14));
        let high_bar = unsafe { config_addr_high.read() };

        Self(u64::from(high_bar) << 32 | u64::from(low_bar & 0xffff_fff0))
    }

    pub fn base_addr(&self) -> PhysAddr {
        PhysAddr::new(self.0)
    }
}

struct BarIndex(u32);
impl BarIndex {
    fn new(index: u32) -> Self {
        assert!(index < 6);
        Self(index)
    }
}
