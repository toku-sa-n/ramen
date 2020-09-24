// SPDX-License-Identifier: GPL-3.0-or-later

use super::{Bus, ConfigAddress, Device, Function, Offset, RawSpace};

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

    pub(super) fn is_xhci(&self) -> bool {
        self.class.base == 0x0c && self.class.sub == 0x03 && self.interface.0 == 0x30
    }

    pub(super) fn has_capability_ptr(&self) -> bool {
        self.status.capability_pointer_exists()
    }

    pub(super) fn is_endpoint(&self) -> bool {
        self.header_type.0 == 0
    }
}

#[derive(Debug)]
struct Id {
    vendor: u16,
    device: u16,
}

impl Id {
    fn fetch(bus: Bus, device: Device) -> Self {
        let config_addr = ConfigAddress::new(bus, device, Function::zero(), Offset::zero());
        let raw_ids = unsafe { config_addr.read() };
        Self {
            vendor: (raw_ids & 0xffff) as u16,
            device: (raw_ids >> 16) as u16,
        }
    }

    fn parse_raw(raw: &RawSpace) -> Self {
        let vendor = (raw.as_slice()[0] & 0xffff) as u16;
        let device = ((raw.as_slice()[0] >> 16) & 0xffff) as u16;

        Self { vendor, device }
    }

    fn valid(&self) -> bool {
        self.vendor != 0xffff
    }
}

#[derive(Debug)]
struct HeaderType(u8);
impl HeaderType {
    fn fetch(bus: Bus, device: Device) -> Self {
        let config_addr = ConfigAddress::new(bus, device, Function::zero(), Offset::new(0x0c));
        let raw = unsafe { config_addr.read() };
        Self((raw >> 16 & 0xff) as u8)
    }

    fn parse_raw(raw: &RawSpace) -> Self {
        let header = ((raw.as_slice()[3] >> 16) & 0xff) as u8;

        Self(header)
    }
}

#[derive(Debug, Copy, Clone)]
struct Status(u16);
impl Status {
    fn fetch(bus: Bus, device: Device) -> Self {
        let config_addr = ConfigAddress::new(bus, device, Function::zero(), Offset::new(0x04));
        let raw = unsafe { config_addr.read() };
        Self((raw >> 16) as u16)
    }

    fn parse_raw(raw: &RawSpace) -> Self {
        let status = ((raw.as_slice()[1] >> 16) & 0xffff) as u16;

        Self(status)
    }

    fn capability_pointer_exists(self) -> bool {
        self.0 & 0b1000 != 0
    }
}

#[derive(Debug)]
struct Class {
    base: u8,
    sub: u8,
}

impl Class {
    fn fetch(bus: Bus, device: Device) -> Self {
        let config_addr = ConfigAddress::new(bus, device, Function::zero(), Offset::new(8));
        let raw_data = unsafe { config_addr.read() };

        Self {
            base: ((raw_data >> 24) & 0xff) as u8,
            sub: ((raw_data >> 16) & 0xff) as u8,
        }
    }

    fn parse_raw(raw: &RawSpace) -> Self {
        let base = ((raw.as_slice()[2] >> 24) & 0xff) as u8;
        let sub = ((raw.as_slice()[2] >> 16) & 0xff) as u8;

        Self { base, sub }
    }
}

#[derive(Debug)]
struct Interface(u8);

impl Interface {
    fn fetch(bus: Bus, device: Device) -> Self {
        let config_addr = ConfigAddress::new(bus, device, Function::zero(), Offset::new(8));
        let raw_data = unsafe { config_addr.read() };

        Self(((raw_data >> 8) & 0xff) as u8)
    }
}
