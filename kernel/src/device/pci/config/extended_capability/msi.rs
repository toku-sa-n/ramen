// SPDX-License-Identifier: GPL-3.0-or-later

use super::{RegisterIndex, Registers};

#[derive(Debug)]
pub struct CapabilitySpec<'a> {
    registers: &'a Registers,
    base: RegisterIndex,
}
