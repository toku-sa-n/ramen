// SPDX-License-Identifier: GPL-3.0-or-later

use super::{Bus, ConfigAddress, Device, Function, Offset};

#[derive(Debug)]
pub(super) struct Common {
    id: Id,
    header_type: HeaderType,
    status: Status,
    class: Class,
    interface: Interface,
}

impl Common {
    pub(super) fn fetch(bus: Bus, device: Device) -> Option<Self> {
        let id = Id::fetch(bus, device);
        if !id.valid() {
            return None;
        }
        let header_type = HeaderType::fetch(bus, device);
        let status = Status::fetch(bus, device);
        let class = Class::fetch(bus, device);
        let interface = Interface::fetch(bus, device);

        Some(Self {
            id,
            header_type,
            status,
            class,
            interface,
        })
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

#[derive(Debug)]
struct Status(u32);
impl Status {
    fn fetch(bus: Bus, device: Device) -> Self {
        let config_addr = ConfigAddress::new(bus, device, Function::zero(), Offset::new(0x04));
        let raw = unsafe { config_addr.read() };
        Self(raw >> 16)
    }

    fn capability_pointer_exists(self) -> bool {
        self.0 & 0b1000 != 0
    }
}

#[derive(Debug)]
struct Class {
    base: u32,
    sub: u32,
}

impl Class {
    fn fetch(bus: Bus, device: Device) -> Self {
        let config_addr = ConfigAddress::new(bus, device, Function::zero(), Offset::new(8));
        let raw_data = unsafe { config_addr.read() };

        Self {
            base: (raw_data >> 24) & 0xff,
            sub: (raw_data >> 16) & 0xff,
        }
    }
}

#[derive(Debug)]
struct Interface(u32);

impl Interface {
    fn fetch(bus: Bus, device: Device) -> Self {
        let config_addr = ConfigAddress::new(bus, device, Function::zero(), Offset::new(8));
        let raw_data = unsafe { config_addr.read() };

        Self((raw_data >> 8) & 0xff)
    }
}
