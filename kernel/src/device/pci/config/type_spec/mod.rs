// SPDX-License-Identifier: GPL-3.0-or-later

mod non_bridge;

use {
    super::{
        bar,
        common::{BridgeType, Common},
        Bar, RegisterIndex, Registers,
    },
    non_bridge::TypeSpecNonBridge,
};

#[derive(Debug)]
pub enum TypeSpec {
    NonBridge(TypeSpecNonBridge),
}

impl TypeSpec {
    pub fn new(raw: &Registers, common: &Common) -> Self {
        match common.bridge_type() {
            BridgeType::NonBridge => TypeSpec::NonBridge(TypeSpecNonBridge::parse_raw(raw)),
            e => panic!("Not implemented: {:?}\ncommon:{:?}", e, common),
        }
    }
}
