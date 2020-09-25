// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{bar, Bus, ConfigAddress, Device, Function, Offset},
    x86_64::PhysAddr,
};

#[derive(Debug, Copy, Clone, Default)]
pub struct Bar(u32);

impl Bar {
    pub(super) fn fetch(bus: Bus, device: Device, bar_index: bar::Index) -> Self {
        let config_addr = ConfigAddress::new(
            bus,
            device,
            Function::zero(),
            Offset::new(0x10 + bar_index.as_u32() * 4),
        );
        Self(unsafe { config_addr.read() })
    }

    pub(super) fn new(bar: u32) -> Self {
        Self(bar)
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

    pub(super) fn as_u32(self) -> u32 {
        self.0
    }

    pub(super) fn base_addr(self, upper: Option<Bar>) -> Option<PhysAddr> {
        if self.ty() == BarType::Bar64Bit && upper.is_some() {
            self.base_addr_64(upper.unwrap())
        } else {
            self.base_addr_32()
        }
    }

    fn base_addr_32(self) -> Option<PhysAddr> {
        match self.ty() {
            BarType::Bar32Bit => Some(PhysAddr::new((self.0 & !0xf) as u64)),
            BarType::Bar64Bit => None,
        }
    }

    fn base_addr_64(self, upper: Bar) -> Option<PhysAddr> {
        match self.ty() {
            BarType::Bar32Bit => None,
            BarType::Bar64Bit => Some(PhysAddr::new(
                (self.0 & !0xf) as u64 | (upper.0 as u64) << 32,
            )),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Index(u32);
impl Index {
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
