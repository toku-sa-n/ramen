// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{Bus, ConfigAddress, Device, Function, Offset},
    x86_64::PhysAddr,
};

#[derive(Debug)]
pub struct Bar(u32);

impl Bar {
    pub(super) fn fetch(bus: Bus, device: Device, bar_index: BarIndex) -> Self {
        let config_addr = ConfigAddress::new(
            bus,
            device,
            Function::zero(),
            Offset::new(0x10 + bar_index.as_u32() * 4),
        );
        let bar = unsafe { config_addr.read() };

        Self(bar)
    }
}

#[derive(Copy, Clone)]
struct BarIndex(u32);
impl BarIndex {
    fn new(index: u32) -> Self {
        assert!(index < 6);
        Self(index)
    }

    fn as_u32(self) -> u32 {
        self.0
    }
}

enum BarType {
    Bar32Bit,
    Bar64Bit,
}
