// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{Bus, ConfigAddress, Device},
    x86_64::PhysAddr,
};

#[derive(Debug)]
pub struct Bar {
    ty: BarType,
    prefetch: bool,
    base_addr: u64,
}

impl Bar {
    pub(super) fn fetch(bus: Bus, device: Device) -> Self {
        let config_addr_low = ConfigAddress::new(bus, device, 0, 0x10);
        let low_bar = unsafe { config_addr_low.read() };

        let config_addr_high = ConfigAddress::new(bus, device, 0, 0x14);
        let high_bar = unsafe { config_addr_high.read() };

        Self {
            ty: if (low_bar >> 1).trailing_zeros() >= 2 {
                BarType::Space32
            } else {
                BarType::Space64
            },
            prefetch: low_bar & 0b100 == 0b100,
            base_addr: u64::from(high_bar) << 32 | u64::from(low_bar & 0xffff_fff0),
        }
    }

    pub fn base_addr(&self) -> PhysAddr {
        PhysAddr::new(self.base_addr)
    }
}

#[derive(Debug)]
enum BarType {
    Space32,
    Space64,
}
