// SPDX-License-Identifier: GPL-3.0-or-later

use super::{Bus, ConfigAddress, Device, Function, Offset};

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
        Self(unsafe { config_addr.read() })
    }

    pub(super) fn ty(&self) -> BarType {
        let ty_raw = self.0 & 0b11;
        if ty_raw == 0 {
            BarType::Bar32Bit
        } else if ty_raw == 0x02 {
            BarType::Bar64Bit
        } else {
            unreachable!();
        }
    }
}

#[derive(Copy, Clone)]
pub struct BarIndex(u32);
impl BarIndex {
    pub fn new(index: u32) -> Self {
        assert!(index < 6);
        Self(index)
    }

    pub(super) fn as_u32(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) enum BarType {
    Bar32Bit,
    Bar64Bit,
}
