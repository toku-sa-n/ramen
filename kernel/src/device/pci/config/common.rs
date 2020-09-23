// SPDX-License-Identifier: GPL-3.0-or-later

use super::{Bus, ConfigAddress, Device, Function, Offset};

#[derive(Debug)]
pub(super) struct Common {
    id: Id,
}

impl Common {
    pub(super) fn fetch(bus: Bus, device: Device) -> Option<Self> {
        let id = Id::fetch(bus, device);
        if !id.is_valid() {
            return None;
        }

        return Some(Self { id });
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

    fn is_valid(&self) -> bool {
        self.vendor != 0xffff
    }
}
