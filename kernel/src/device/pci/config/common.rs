// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{RegisterIndex, Registers},
    core::convert::TryFrom,
};

#[derive(Debug)]
pub struct Common<'a> {
    registers: &'a Registers,
}

impl<'a> Common<'a> {
    pub fn new(registers: &'a Registers) -> Self {
        Self { registers }
    }

    pub fn is_xhci(&self) -> bool {
        self.class().is_xhci()
    }

    pub fn has_capability_ptr(&self) -> bool {
        self.status().capability_pointer_exists()
    }

    pub fn bridge_type(&self) -> BridgeType {
        self.header_type().bridge_type()
    }

    fn class(&self) -> Class {
        Class::new(self.registers)
    }

    fn status(&self) -> Status {
        Status::parse_raw(self.registers)
    }

    fn header_type(&self) -> HeaderType {
        HeaderType::new(self.registers)
    }
}

#[derive(Debug)]
struct Id {
    vendor: u16,
    device: u16,
}

impl Id {
    fn new(raw: &Registers) -> Self {
        let vendor = u16::try_from(raw.get(RegisterIndex::zero()) & 0xffff).unwrap();
        let device = u16::try_from((raw.get(RegisterIndex::zero()) >> 16) & 0xffff).unwrap();

        Self { vendor, device }
    }
}

#[derive(Debug, Copy, Clone)]
struct HeaderType(u8);
impl HeaderType {
    fn new(register: &Registers) -> Self {
        let header = u8::try_from((register.get(RegisterIndex::new(3)) >> 16) & 0xff).unwrap();

        Self(header)
    }

    fn bridge_type(self) -> BridgeType {
        match self.0 & 0x7f {
            0 => BridgeType::NonBridge,
            1 => BridgeType::PciToPci,
            2 => BridgeType::PciToCardbus,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub enum BridgeType {
    NonBridge,
    PciToPci,
    PciToCardbus,
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
struct Class<'a> {
    registers: &'a Registers,
}

impl<'a> Class<'a> {
    fn is_xhci(&self) -> bool {
        self.base() == 0x0c && self.sub() == 0x03 && self.interface() == 0x30
    }

    fn new(registers: &'a Registers) -> Self {
        Self { registers }
    }

    fn base(&self) -> u8 {
        ((self.registers.get(RegisterIndex::new(2)) >> 24) & 0xff) as u8
    }

    fn sub(&self) -> u8 {
        ((self.registers.get(RegisterIndex::new(2)) >> 16) & 0xff) as u8
    }

    fn interface(&self) -> u8 {
        ((self.registers.get(RegisterIndex::new(2)) >> 8) & 0xff) as u8
    }
}
