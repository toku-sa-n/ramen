// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{Bus, ConfigAddress, Device, Function, Offset},
    x86_64::PhysAddr,
};

#[derive(Debug)]
pub struct Bar {
    ty: BarType,
    base: PhysAddr,
}

impl Bar {
    pub(super) fn fetch(bus: Bus, device: Device, bar_index: BarIndex) -> Self {
        let config_addr = ConfigAddress::new(
            bus,
            device,
            Function::zero(),
            Offset::new(0x10 + bar_index.as_u32() * 4),
        );
        let raw = unsafe { config_addr.read() };
        let ty = {
            let ty_raw = (raw >> 1) & 0b11;
            if ty_raw == 0 {
                BarType::Bar32Bit
            } else if ty_raw == 0x02 {
                BarType::Bar64Bit
            } else {
                unreachable!();
            }
        };
        let base = PhysAddr::new(raw as u64 & !0xf);

        Self { ty, base }
    }
}

#[derive(Copy, Clone)]
pub(super) struct BarIndex(u32);
impl BarIndex {
    pub(super) fn new(index: u32) -> Self {
        assert!(index < 6);
        Self(index)
    }

    fn as_u32(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Copy, Clone)]
enum BarType {
    Bar32Bit,
    Bar64Bit,
}
