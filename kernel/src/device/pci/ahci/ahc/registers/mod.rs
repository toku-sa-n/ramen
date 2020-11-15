// SPDX-License-Identifier: GPL-3.0-or-later

pub mod generic;
pub mod port;

use {super::AchiBaseAddr, alloc::vec::Vec, generic::Generic};

pub struct Registers {
    pub generic: Generic,
    pub port_regs: Vec<Option<port::Registers>>,
}
impl Registers {
    pub fn new(abar: AchiBaseAddr) -> Self {
        let generic = Generic::new(abar);
        let port_regs = Self::collect_port_regs(abar, &generic);

        Self { generic, port_regs }
    }

    fn collect_port_regs(abar: AchiBaseAddr, generic: &Generic) -> Vec<Option<port::Registers>> {
        (0..32)
            .map(|i| port::Registers::new(abar, i, generic))
            .collect()
    }
}
