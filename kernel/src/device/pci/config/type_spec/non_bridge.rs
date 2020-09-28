// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{bar, Bar, RegisterIndex, Registers},
    x86_64::PhysAddr,
};

#[derive(Debug)]
pub struct TypeSpec {
    bars: [Bar; 6],
}

impl TypeSpec {
    pub(super) fn new(raw: &Registers) -> Self {
        let mut bars = [Bar::default(); 6];
        for (i, bar) in bars.iter_mut().enumerate() {
            *bar = Bar::new(raw.get(RegisterIndex::new(i + 4)));
        }

        Self { bars }
    }

    pub fn base_addr(&self, index: bar::Index) -> PhysAddr {
        let index = index.as_usize();
        let upper = if index == 5 {
            None
        } else {
            Some(self.bars[index + 1])
        };

        self.bars[index].base_addr(upper).unwrap()
    }
}
