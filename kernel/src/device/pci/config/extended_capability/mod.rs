// SPDX-License-Identifier: GPL-3.0-or-later

pub mod msi;
pub mod msi_x;

use {
    super::{RegisterIndex, Registers, TypeSpec},
    crate::accessor::single_object,
    alloc::boxed::Box,
    bitfield::bitfield,
    common::constant::LOCAL_APIC_ID_REGISTER_ADDR,
    core::{
        convert::{From, TryFrom},
        iter::Iterator,
    },
    msi::Msi,
    msi_x::MsiX,
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
    pub fn init_for_xhci(&self, space_type_spec: &TypeSpec) -> Result<(), Error> {
        match self.capability_spec() {
            None => Err(Error::MsiAndMsiXNotFound),
            Some(spec) => {
                info!("MSI or MSI-X found.");
                spec.init_for_xhci(space_type_spec);
                Ok(())
            }
        }
    }

    fn capability_spec(&self) -> Option<Box<dyn CapabilitySpec + 'a>> {
        match self.ty() {
            Some(ty) => Some(match ty {
                CapabilityType::Msi => Box::new(Msi::new(self.registers, self.base)),
                CapabilityType::MsiX => Box::new(MsiX::new(self.registers, self.base)),
            }),
            None => None,
        }
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

pub enum Error {
    MsiAndMsiXNotFound,
}

pub trait CapabilitySpec {
    fn init_for_xhci(&self, space_type_spec: &TypeSpec);
}

fn get_local_apic_id() -> u8 {
    let accessor = single_object::Accessor::<u32>::new(LOCAL_APIC_ID_REGISTER_ADDR, 0);
    u8::try_from(*accessor >> 24).unwrap()
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
        } else if self.0 == 0x05 {
            Some(CapabilityType::Msi)
        } else {
            None
        }
    }
}

enum CapabilityType {
    MsiX,
    Msi,
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

bitfield! {
    pub struct MessageAddress(u32);

    pub redirection_hint, set_redirection_hint: 3;
    pub u8, destination_id, set_destination_id: 19, 12;
}

impl From<MessageAddress> for u32 {
    fn from(address: MessageAddress) -> Self {
        address.0
    }
}

impl From<u32> for MessageAddress {
    fn from(val: u32) -> Self {
        Self(val)
    }
}

bitfield! {
    pub struct MessageData(u32);

    pub vector, set_vector: 7, 0;
    pub delivery_mode, set_delivery_mode: 10, 8;
    level, set_level: 14;
    trigger_mode, set_trigger_mode: 15;
}

impl MessageData {
    pub fn set_level_trigger(&mut self) {
        self.set_trigger_mode(true);
        self.set_level(true);
    }
}

impl From<MessageData> for u32 {
    fn from(address: MessageData) -> Self {
        address.0
    }
}

impl From<u32> for MessageData {
    fn from(val: u32) -> Self {
        Self(val)
    }
}
