// SPDX-License-Identifier: GPL-3.0-or-later

pub mod usb_legacy_support;

use core::convert::TryInto;

use super::capability::Capability;
use crate::mem::accessor::Accessor;
use os_units::Bytes;
use usb_legacy_support::UsbLegacySupport;
use x86_64::PhysAddr;

pub struct List {
    head: PhysAddr,
}
impl List {
    /// SAFETY: Caller must ensure that `addr` points to the head of a xHCI extended capability.
    pub unsafe fn new(mmio_base: PhysAddr, capability: &Capability) -> Option<Self> {
        let p1 = capability.hc_cp_params_1.read();
        let xecp = p1.xhci_extended_capabilities_pointer();

        if xecp > 0 {
            let head = mmio_base + u64::from(xecp << 2);

            Some(Self { head })
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = ExtendedCapability> {
        Iter {
            addr: Some(self.head),
        }
    }
}

struct Iter {
    addr: Option<PhysAddr>,
}
impl Iter {
    fn next_addr(&self) -> Option<PhysAddr> {
        let offset = self.header()? >> 8;
        if offset == 0 {
            None
        } else {
            Some(self.addr? + u64::from(offset << 2))
        }
    }

    fn id(&self) -> Option<u8> {
        Some((self.header()? & 0xff).try_into().unwrap())
    }

    fn header(&self) -> Option<u32> {
        // SAFETY: This is safe because `self.addr` points to the head of an extended capability.
        let a: Accessor<u32> = unsafe { Accessor::user(self.addr?, Bytes::zero()) };
        Some(a.read())
    }
}
impl Iterator for Iter {
    type Item = ExtendedCapability;

    fn next(&mut self) -> Option<Self::Item> {
        let a = self.addr?;
        let next_addr = self.next_addr();

        let item = if let Some(1) = self.id() {
            let a = unsafe { Accessor::user(a, Bytes::zero()) };
            ExtendedCapability::UsbLegacySupport(a)
        } else {
            ExtendedCapability::UnImplemented
        };

        self.addr = next_addr;

        Some(item)
    }
}

pub enum ExtendedCapability {
    UsbLegacySupport(Accessor<UsbLegacySupport>),
    UnImplemented,
}
