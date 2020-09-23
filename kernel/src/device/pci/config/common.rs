// SPDX-License-Identifier: GPL-3.0-or-later

use super::{Bus, ConfigAddress, Device, Function, Offset};

#[derive(Debug)]
pub(super) struct Common {
    id: Id,
    header_type: HeaderType,
}

impl Common {
    pub(super) fn fetch(bus: Bus, device: Device) -> Option<Self> {
        let id = Id::fetch(bus, device);
        if !id.valid() {
            return None;
        }
        let header_type = HeaderType::fetch(bus, device);

        Some(Self { id, header_type })
    }
}

#[derive(Debug)]
struct Id {
    vendor: u32,
    device: u32,
}

impl Id {
    fn fetch(bus: Bus, device: Device) -> Self {
        let config_addr = ConfigAddress::new(bus, device, Function::zero(), Offset::zero());
        let raw_ids = unsafe { config_addr.read() };
        Self {
            vendor: raw_ids & 0xffff,
            device: raw_ids >> 16,
        }
    }

    fn valid(&self) -> bool {
        self.vendor != 0xffff
    }
}

#[derive(Debug)]
struct HeaderType(u32);
impl HeaderType {
    fn fetch(bus: Bus, device: Device) -> Self {
        let config_addr = ConfigAddress::new(bus, device, Function::zero(), Offset::new(0x08));
        let raw = unsafe { config_addr.read() };
        Self(raw >> 16 & 0xff)
    }
}
