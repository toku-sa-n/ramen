// SPDX-License-Identifier: GPL-3.0-or-later

pub mod command;
pub mod event;
mod raw;
pub mod transfer;
pub mod trb;

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct CycleBit(bool);
impl CycleBit {
    fn new(val: bool) -> Self {
        Self(val)
    }

    fn toggle(&mut self) {
        self.0 = !self.0;
    }
}
impl From<CycleBit> for bool {
    fn from(cycle_bit: CycleBit) -> Self {
        cycle_bit.0
    }
}
