// SPDX-License-Identifier: GPL-3.0-or-later

mod msi_x;

use {
    super::{Common, Offset, RawSpace, TypeSpec},
    alloc::vec::Vec,
    msi_x::CapabilitySpecMsiX,
};

#[derive(Debug)]
pub struct ExtendedCapabilities<'a>(Vec<ExtendedCapability<'a>>);

impl<'a> ExtendedCapabilities<'a> {
    pub fn new(raw: &RawSpace, common: &Common, type_spec: &TypeSpec) -> Option<Self> {
        let mut base = Self::parse_raw_to_get_capability_ptr(raw, common)?;
        let mut capabilities = Vec::new();

        while {
            let extended_capability = ExtendedCapability::new(&raw, base, type_spec);
            base = extended_capability.next_ptr();
            info!("Extended Capability: {:?}", extended_capability);
            capabilities.push(extended_capability);

            !base.is_null()
        } {}

        Some(Self(capabilities))
    }

    fn parse_raw_to_get_capability_ptr(raw: &RawSpace, common: &Common) -> Option<Offset> {
        if common.has_capability_ptr() {
            Some(Offset::new(raw.as_slice()[0x0d] & 0xfc))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct ExtendedCapability<'a> {
    id: Id,
    next_ptr: Offset,
    capability_spec: Option<CapabilitySpec<'a>>,
}

impl<'a> ExtendedCapability<'a> {
    fn new(raw: &RawSpace, offset: Offset, type_spec: &TypeSpec) -> Self {
        let id = Id::parse_raw(raw, offset);
        let next_ptr = Offset::new((raw[offset] >> 8) & 0xff);
        let capability_spec = CapabilitySpec::new(raw, offset, id, type_spec);

        Self {
            id,
            next_ptr,
            capability_spec,
        }
    }

    fn next_ptr(&self) -> Offset {
        self.next_ptr
    }
}

#[derive(Debug)]
enum CapabilitySpec<'a> {
    MsiX(CapabilitySpecMsiX<'a>),
}

impl<'a> CapabilitySpec<'a> {
    fn new(raw: &RawSpace, offset: Offset, id: Id, type_spec: &TypeSpec) -> Option<Self> {
        if id.0 == 0x11 {
            Some(Self::MsiX(CapabilitySpecMsiX::new(raw, offset, type_spec)))
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Id(u8);
impl Id {
    fn parse_raw(raw: &RawSpace, offset: Offset) -> Self {
        Self((raw[offset] & 0xff) as u8)
    }
}
