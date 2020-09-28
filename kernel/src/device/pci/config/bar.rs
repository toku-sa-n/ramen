// SPDX-License-Identifier: GPL-3.0-or-later

use x86_64::PhysAddr;

#[derive(Debug, Copy, Clone, Default)]
pub struct Bar(u32);

impl Bar {
    pub(super) fn new(bar: u32) -> Self {
        Self(bar)
    }

    pub(super) fn ty(self) -> BarType {
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
        match upper {
            Some(upper) => match self.ty() {
                BarType::Bar64Bit => self.base_addr_64(upper),
                BarType::Bar32Bit => self.base_addr_32(),
            },
            None => self.base_addr_32(),
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

    pub(super) fn as_usize(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) enum BarType {
    Bar32Bit,
    Bar64Bit,
}
