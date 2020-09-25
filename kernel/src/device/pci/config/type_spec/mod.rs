// SPDX-License-Identifier: GPL-3.0-or-later

mod non_bridge;

use {
    super::{Bar, Common, RawSpace},
    non_bridge::TypeSpecNonBridge,
};

enum TypeSpec {
    NonBridge(non_bridge::TypeSpecNonBridge),
}

impl TypeSpec {
    fn parse_raw(raw: &RawSpace, common: &Common) -> Self {
        match common.header_type() {
            0 => TypeSpec::NonBridge(TypeSpecNonBridge::parse_raw(raw)),
            _ => todo!(),
        }
    }
}
