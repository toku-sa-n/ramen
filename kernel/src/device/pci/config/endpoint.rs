// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{Bar, BarIndex, BarType, Bus, Device},
    x86_64::PhysAddr,
};

#[derive(Debug)]
pub(super) struct EndPoint {
    bar: [Bar; 6],
}

impl EndPoint {
    pub(super) fn fetch(bus: Bus, device: Device) -> Self {
        let bar: [Bar; 6];
        for i in 0..6u32 {
            bar[i as usize] = Bar::fetch(bus, device, BarIndex::new(i));
        }

        Self { bar }
    }

    pub fn base_addr(&self, index: BarIndex) -> PhysAddr {
        let index = index.as_u32() as usize;

        if index == 5 && self.bar[index].ty() == BarType::Bar64Bit {
            panic!("Attempt to get the 5th Base Address Register, whose type is 64-bit!");
        }

        let addr = match self.bar[index].ty() {
            BarType::Bar32Bit => (self.bar[index].as_u32() & !0xf) as u64,
            BarType::Bar64Bit => {
                (self.bar[index + 1].as_u32() << 32) as u64 & self.bar[index].as_u32() as u64
            }
        };

        PhysAddr::new(addr)
    }
}
