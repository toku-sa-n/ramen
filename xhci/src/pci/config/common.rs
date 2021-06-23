// SPDX-License-Identifier: GPL-3.0-or-later

use super::{extended_capability, RegisterIndex, Registers};
use bit_field::BitField;
use core::convert::{TryFrom, TryInto};

#[derive(Debug)]
pub(super) struct Common<'a> {
    registers: &'a Registers,
}
impl<'a> Common<'a> {
    pub(super) fn new(registers: &'a Registers) -> Self {
        Self { registers }
    }

    pub(super) fn is_xhci(&self) -> bool {
        self.class().is_xhci()
    }

    pub(super) fn bridge_type(&self) -> BridgeType {
        self.header_type().bridge_type()
    }

    pub(super) fn iter_capability_list(&self) -> Option<impl Iterator<Item = u8> + '_> {
        self.capability_list_exists().then(|| {
            let ptr = self.capability_pointer().get();
            let index = RegisterIndex::new((ptr >> 2).into());

            extended_capability::Iter::new(self.registers, index)
        })
    }

    pub(super) fn capability_list_exists(&self) -> bool {
        self.status().capability_list_exists()
    }

    fn class(&self) -> Class<'_> {
        Class::new(self.registers)
    }

    fn header_type(&self) -> HeaderType {
        HeaderType::new(self.registers)
    }

    fn status(&self) -> Status {
        Status::new(self.registers)
    }

    fn capability_pointer(&self) -> CapabilityPointer {
        CapabilityPointer::new(self.registers)
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
pub(super) enum BridgeType {
    NonBridge,
    PciToPci,
    PciToCardbus,
}

#[derive(Debug)]
struct Class<'a> {
    registers: &'a Registers,
}
impl<'a> Class<'a> {
    fn new(registers: &'a Registers) -> Self {
        Self { registers }
    }

    fn is_xhci(&self) -> bool {
        self.as_tuple() == (0x0c, 0x03, 0x30)
    }

    fn as_tuple(&self) -> (u8, u8, u8) {
        (self.base(), self.sub(), self.interface())
    }

    fn base(&self) -> u8 {
        self.raw().get_bits(24..=31).try_into().unwrap()
    }

    fn sub(&self) -> u8 {
        self.raw().get_bits(16..=23).try_into().unwrap()
    }

    fn interface(&self) -> u8 {
        self.raw().get_bits(8..=15).try_into().unwrap()
    }

    fn raw(&self) -> u32 {
        self.registers.get(RegisterIndex::new(2))
    }
}

#[derive(Debug)]
struct Status<'a> {
    registers: &'a Registers,
}
impl<'a> Status<'a> {
    fn new(registers: &'a Registers) -> Self {
        Self { registers }
    }

    fn capability_list_exists(&self) -> bool {
        self.raw().get_bit(4)
    }

    fn raw(&self) -> u16 {
        let raw = self.registers.get(RegisterIndex::new(1)).get_bits(16..=31);

        raw.try_into().unwrap()
    }
}

#[derive(Debug)]
struct CapabilityPointer<'a> {
    registers: &'a Registers,
}
impl<'a> CapabilityPointer<'a> {
    fn new(registers: &'a Registers) -> Self {
        Self { registers }
    }

    fn get(&self) -> u8 {
        let raw = self.registers.get(RegisterIndex::new(0xd)).get_bits(0..=7);

        raw.try_into().unwrap()
    }
}
