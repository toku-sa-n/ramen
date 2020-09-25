// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{bar, Bar, RawSpace},
    x86_64::PhysAddr,
};

#[derive(Debug)]
pub struct TypeSpecNonBridge {
    bar: [Bar; 6],
}

impl TypeSpecNonBridge {
    pub(super) fn parse_raw(raw: &RawSpace) -> Self {
        let mut bar = [Bar::default(); 6];
        for i in 0..6 {
            bar[i] = Bar::new(raw.as_slice()[i + 4]);
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
