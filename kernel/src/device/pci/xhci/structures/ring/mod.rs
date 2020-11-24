// SPDX-License-Identifier: GPL-3.0-or-later

pub mod command;
pub mod event;
pub mod transfer;

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

#[macro_export]
macro_rules! add_trb {
    ($t:ident) => {
        #[repr(transparent)]
        #[derive(Debug)]
        pub struct $t([u32; 4]);
        impl $t {
            #[allow(dead_code)]
            fn set_cycle_bit(&mut self, c: crate::device::pci::xhci::structures::ring::CycleBit) {
                use bit_field::BitField;
                self.0[3].set_bit(0, c.into());
            }

            #[allow(dead_code)]
            fn set_trb_type(&mut self, t: u8) {
                use bit_field::BitField;
                self.0[3].set_bits(10..=15, t.into());
            }
        }
    };
}
