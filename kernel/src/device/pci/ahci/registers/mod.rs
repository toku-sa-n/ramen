// SPDX-License-Identifier: GPL-3.0-or-later

pub mod generic;
pub mod port;

use super::AchiBaseAddr;
use alloc::vec::Vec;
use generic::Generic;

pub struct Registers {
    pub generic: Generic,
    pub port_regs: Vec<Option<port::Registers>>,
}
impl Registers {
    /// SAFETY: This method is unsafe because if `abar` is not the valid AHCI base address, it can
    /// violate memory safety.
    pub unsafe fn new(abar: AchiBaseAddr) -> Self {
        let generic = Generic::new(abar);
        let port_regs = Self::collect_port_regs(abar, &generic);

        Self { generic, port_regs }
    }

    /// SAFETY: This method is unsafe because if `abar` is not the valid AHCI base address, it can
    /// violate memory safety.
    unsafe fn collect_port_regs(
        abar: AchiBaseAddr,
        generic: &Generic,
    ) -> Vec<Option<port::Registers>> {
        (0..32)
            .map(|i| port::Registers::new(abar, i, generic))
            .collect()
    }
}
