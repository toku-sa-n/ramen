// SPDX-License-Identifier: GPL-3.0-or-later

mod non_bridge;

use {
    super::{bar, Bar, Common, Registers},
    non_bridge::TypeSpecNonBridge,
};

#[derive(Debug)]
pub enum TypeSpec {
    NonBridge(TypeSpecNonBridge),
}

impl TypeSpec {
    pub fn parse_raw(raw: &Registers, common: &Common) -> Self {
        match common.header_type() & !0b10000000 {
            0 => TypeSpec::NonBridge(TypeSpecNonBridge::parse_raw(raw)),
            e => panic!("Not implemented: {}\ncommon:{:?}", e, common),
        }
    }
}
