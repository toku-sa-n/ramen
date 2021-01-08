// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::{TryFrom, TryInto};
use os_units::Bytes;
use x86_64::PhysAddr;

pub mod command;
pub mod event;
pub mod transfer;

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct CycleBit(bool);
impl CycleBit {
    pub fn new(val: bool) -> Self {
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
    ($t:ident,$id:expr) => {
        #[repr(transparent)]
        #[derive(Copy, Clone, Debug)]
        pub struct $t(pub [u32; 4]);
        impl $t {
            const ID: u8 = $id;

            #[allow(dead_code)]
            fn set_cycle_bit(&mut self, c: crate::device::pci::xhci::structures::ring::CycleBit) {
                use bit_field::BitField;
                self.0[3].set_bit(0, c.into());
            }

            #[allow(dead_code)]
            fn set_trb_type(&mut self) {
                use bit_field::BitField;
                self.0[3].set_bits(10..=15, Self::ID.into());
            }
        }
    };
}

#[macro_export]
macro_rules! impl_default_simply_adds_trb_id {
    ($t:ident) => {
        impl Default for $t {
            fn default() -> Self {
                let mut t = Self([0; 4]);
                t.set_trb_type();
                t
            }
        }
    };
}

add_trb!(Link, 6);
impl Link {
    const SIZE: Bytes = Bytes::new(16);

    fn new(addr_to_ring: PhysAddr) -> Self {
        assert!(addr_to_ring.is_aligned(u64::try_from(Self::SIZE.as_usize()).unwrap()));
        let mut trb = Self([0; 4]);
        trb.set_trb_type();
        trb.set_addr(addr_to_ring.as_u64());
        trb
    }

    fn set_addr(&mut self, a: u64) {
        let l = a & 0xffff_ffff;
        let u = a >> 32;
        self.0[0] = l.try_into().unwrap();
        self.0[1] = u.try_into().unwrap();
    }
}
