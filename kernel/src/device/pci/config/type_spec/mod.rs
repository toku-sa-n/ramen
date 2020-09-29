// SPDX-License-Identifier: GPL-3.0-or-later

mod non_bridge;

use {
    super::{
        bar,
        common::{BridgeType, Common},
        Bar, RegisterIndex, Registers,
    },
    x86_64::PhysAddr,
};

#[derive(Debug)]
pub enum TypeSpec<'a> {
    NonBridge(non_bridge::TypeSpec<'a>),
}

impl<'a> TypeSpec<'a> {
    pub fn new(registers: &'a Registers, common: &Common) -> Self {
        match common.bridge_type() {
            BridgeType::NonBridge => TypeSpec::NonBridge(non_bridge::TypeSpec::new(registers)),
            e => panic!("Not implemented: {:?}\ncommon:{:?}", e, common),
        }
    }

    pub fn base_address(&self, index: bar::Index) -> PhysAddr {
        let TypeSpec::NonBridge(non_bridge) = self;
        non_bridge.base_addr(index)
    }
}
