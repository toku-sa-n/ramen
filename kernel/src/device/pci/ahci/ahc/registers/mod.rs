// SPDX-License-Identifier: GPL-3.0-or-later

pub mod generic;

use {super::AchiBaseAddr, generic::Generic};

pub struct Registers {
    pub generic: Generic,
}
impl Registers {
    pub fn new(abar: AchiBaseAddr) -> Self {
        Self {
            generic: Generic::new(abar),
        }
    }
}
