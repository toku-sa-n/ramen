// SPDX-License-Identifier: GPL-3.0-or-later

pub mod msi_x;

use {
    super::{RegisterIndex, Registers},
    core::{
        convert::{From, TryFrom},
        iter::Iterator,
    },
};

pub struct Iter<'a> {
    registers: &'a Registers,
    base: RegisterIndex,
}
impl<'a> Iter<'a> {
    pub fn new(registers: &'a Registers, base: RegisterIndex) -> Self {
        Self { registers, base }
    }
}
impl<'a> Iterator for Iter<'a> {
    type Item = ExtendedCapability<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.base.is_null() {
            None
        } else {
            let extended_capability = ExtendedCapability::new(self.registers, self.base);
            let next_pointer = extended_capability.next_ptr();
            self.base = next_pointer.into();

            Some(extended_capability)
        }
    }
}

#[derive(Debug)]
pub struct ExtendedCapability<'a> {
    registers: &'a Registers,
    base: RegisterIndex,
}

impl<'a> ExtendedCapability<'a> {
    pub fn capability_spec(&self) -> Option<CapabilitySpec> {
        CapabilitySpec::new(&self.registers, self.base, &self)
    }

    fn new(registers: &'a Registers, base: RegisterIndex) -> Self {
        Self { registers, base }
    }

    fn next_ptr(&self) -> NextPointer {
        NextPointer::new(self.registers, self.base)
    }

    fn ty(&self) -> Option<CapabilityType> {
        self.id().ty()
    }

    fn id(&self) -> Id {
        Id::new(self.registers, self.base)
    }
}

#[derive(Debug)]
pub enum CapabilitySpec<'a> {
    MsiX(msi_x::CapabilitySpec<'a>),
}

impl<'a> CapabilitySpec<'a> {
    fn new(
        registers: &'a Registers,
        base: RegisterIndex,
        extended_capability: &ExtendedCapability,
    ) -> Option<Self> {
        if extended_capability.ty().is_some() {
            Some(Self::MsiX(msi_x::CapabilitySpec::new(registers, base)))
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Id(u8);
impl Id {
    fn new(registers: &Registers, base: RegisterIndex) -> Self {
        Self(u8::try_from(registers.get(base) & 0xff).unwrap())
    }

    fn ty(self) -> Option<CapabilityType> {
        if self.0 == 0x11 {
            Some(CapabilityType::MsiX)
        } else {
            None
        }
    }
}

enum CapabilityType {
    MsiX,
}

#[derive(Debug)]
struct NextPointer(RegisterIndex);
impl NextPointer {
    fn new(registers: &Registers, base: RegisterIndex) -> Self {
        Self(RegisterIndex::new(
            usize::try_from(((registers.get(base) >> 8) & 0xff) >> 2).unwrap(),
        ))
    }
}
impl From<NextPointer> for RegisterIndex {
    fn from(next_pointer: NextPointer) -> Self {
        next_pointer.0
    }
}
