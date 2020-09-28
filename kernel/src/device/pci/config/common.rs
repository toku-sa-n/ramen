// SPDX-License-Identifier: GPL-3.0-or-later

use super::{RegisterIndex, Registers};

#[derive(Debug)]
pub struct Common {
    id: Id,
    header_type: HeaderType,
    status: Status,
    class: Class,
    interface: Interface,
}

impl Common {
    pub(super) fn new(raw: &Registers) -> Self {
        let id = Id::parse_raw(raw);
        let header_type = HeaderType::parse_raw(raw);
        let status = Status::parse_raw(raw);
        let class = Class::parse_raw(raw);
        let interface = Interface::parse_raw(raw);

        Self {
            id,
            header_type,
            status,
            class,
            interface,
        }
    }

    pub(super) fn is_xhci(&self) -> bool {
        self.class.base == 0x0c && self.class.sub == 0x03 && self.interface.0 == 0x30
    }

    pub(super) fn has_capability_ptr(&self) -> bool {
        self.status.capability_pointer_exists()
    }

    pub(super) fn header_type(&self) -> u8 {
        self.header_type.as_u8()
    }
}

#[derive(Debug)]
struct Id {
    vendor: u16,
    device: u16,
}

impl Id {
    fn parse_raw(raw: &Registers) -> Self {
        let vendor = (raw.get(RegisterIndex::zero()) & 0xffff) as u16;
        let device = ((raw.get(RegisterIndex::zero()) >> 16) & 0xffff) as u16;

        Self { vendor, device }
    }
}

#[derive(Debug, Copy, Clone)]
struct HeaderType(u8);
impl HeaderType {
    fn parse_raw(raw: &Registers) -> Self {
        let header = ((raw.get(RegisterIndex::new(3)) >> 16) & 0xff) as u8;

        Self(header)
    }

    fn as_u8(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Copy, Clone)]
struct Status(u16);
impl Status {
    fn parse_raw(raw: &Registers) -> Self {
        let status = ((raw.get(RegisterIndex::new(1)) >> 16) & 0xffff) as u16;

        Self(status)
    }

    fn capability_pointer_exists(self) -> bool {
        self.0 & 0b10000 != 0
    }
}

#[derive(Debug)]
struct Class {
    base: u8,
    sub: u8,
}

impl Class {
    fn parse_raw(raw: &Registers) -> Self {
        let base = ((raw.get(RegisterIndex::new(2)) >> 24) & 0xff) as u8;
        let sub = ((raw.get(RegisterIndex::new(2)) >> 16) & 0xff) as u8;

        Self { base, sub }
    }
}

#[derive(Debug)]
struct Interface(u8);

impl Interface {
    fn parse_raw(raw: &Registers) -> Self {
        let interface = ((raw.get(RegisterIndex::new(2)) >> 8) & 0xff) as u8;

        Self(interface)
    }
}
