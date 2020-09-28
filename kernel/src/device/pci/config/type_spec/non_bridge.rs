// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{bar, Bar, RegisterIndex, Registers},
    x86_64::PhysAddr,
};

#[derive(Debug)]
pub struct TypeSpec {
    bar: [Bar; 6],
}

impl TypeSpec {
    pub(super) fn new(raw: &Registers) -> Self {
        let mut bar = [Bar::default(); 6];
        for i in 0..6 {
            bar[i] = Bar::new(raw.get(RegisterIndex::new(i + 4)));
        }

        Self { bar }
    }

    pub fn base_addr(&self, index: bar::Index) -> PhysAddr {
        let index = index.as_usize();
        let upper = if index == 5 {
            None
        } else {
            Some(self.bar[index + 1])
        };

        self.bar[index].base_addr(upper).unwrap()
    }
}
