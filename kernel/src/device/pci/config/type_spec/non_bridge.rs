// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{bar, Bar, RegisterIndex, Registers},
    x86_64::PhysAddr,
};

#[derive(Debug)]
pub struct TypeSpec<'a> {
    registers: &'a Registers,
}

impl<'a> TypeSpec<'a> {
    pub fn new(registers: &'a Registers) -> Self {
        Self { registers }
    }

    pub fn base_addr(&self, index: bar::Index) -> PhysAddr {
        let upper = if index == bar::Index::new(5) {
            None
        } else {
            Some(self.bar(index + 1))
        };

        for i in 0..6 {
            info!("Bar{}: {:?}", i, self.bar(bar::Index::new(i)));
        }

        self.bar(index)
            .base_addr(upper)
            .expect("Could not calculate Base Address.")
    }

    fn bar(&self, index: bar::Index) -> Bar {
        Bar::new(self.registers.get(RegisterIndex::from(index)))
    }
}
