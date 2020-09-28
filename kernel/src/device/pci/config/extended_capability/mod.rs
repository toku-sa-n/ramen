// SPDX-License-Identifier: GPL-3.0-or-later

mod msi_x;

use {
    super::{Common, RegisterIndex, Registers, TypeSpec},
    alloc::vec::Vec,
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
    fn new(registers: &'a Registers, base: RegisterIndex) -> Self {
        // let id = Id::parse_raw(raw, base);
        // let next_ptr = RegisterIndex::new(((raw.get(base) >> 8) & 0xff) as usize);
        // let capability_spec = CapabilitySpec::new(raw, base, id, type_spec);

        Self { registers, base }
    }

    fn next_ptr(&self) -> NextPointer {
        NextPointer::new(self.registers, self.base)
    }
}

#[derive(Debug)]
enum CapabilitySpec<'a> {
    MsiX(msi_x::CapabilitySpec<'a>),
}

impl<'a> CapabilitySpec<'a> {
    fn new(raw: &Registers, offset: RegisterIndex, id: Id, type_spec: &TypeSpec) -> Option<Self> {
        if id.0 == 0x11 {
            Some(Self::MsiX(msi_x::CapabilitySpec::new(
                raw, offset, type_spec,
            )))
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Id(u8);
impl Id {
    fn parse_raw(raw: &Registers, offset: RegisterIndex) -> Self {
        Self(u8::try_from(raw.get(offset) & 0xff).unwrap())
    }
}

struct NextPointer(RegisterIndex);
impl NextPointer {
    fn new(registers: &Registers, base: RegisterIndex) -> Self {
        Self(RegisterIndex::new(
            usize::try_from((registers.get(base) >> 8) & 0xff).unwrap(),
        ))
    }

    fn as_register_index(&self) -> RegisterIndex {
        self.0
    }
}
impl From<NextPointer> for RegisterIndex {
    fn from(next_pointer: NextPointer) -> Self {
        next_pointer.0
    }
}
