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

    pub fn is_ahci(&self) -> bool {
        self.class().is_ahci()
    }

    pub fn bridge_type(&self) -> BridgeType {
        self.header_type().bridge_type()
    }

    fn class(&self) -> Class {
        Class::new(self.registers)
    }

    fn header_type(&self) -> HeaderType {
        HeaderType::new(self.registers)
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

#[derive(Debug)]
struct Class<'a> {
    registers: &'a Registers,
}

impl<'a> Class<'a> {
    fn is_xhci(&self) -> bool {
        self.base() == 0x0c && self.sub() == 0x03 && self.interface() == 0x30
    }

    fn is_ahci(&self) -> bool {
        self.base() == 0x01 && self.sub() == 0x06
    }

    fn new(registers: &'a Registers) -> Self {
        Self { registers }
    }

    fn base(&self) -> u8 {
        u8::try_from((self.registers.get(RegisterIndex::new(2)) >> 24) & 0xff).unwrap()
    }

    fn sub(&self) -> u8 {
        u8::try_from((self.registers.get(RegisterIndex::new(2)) >> 16) & 0xff).unwrap()
    }

    fn interface(&self) -> u8 {
        u8::try_from((self.registers.get(RegisterIndex::new(2)) >> 8) & 0xff).unwrap()
    }
}
