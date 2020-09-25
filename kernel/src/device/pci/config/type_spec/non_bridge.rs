// SPDX-License-Identifier: GPL-3.0-or-later

use super::{Bar, RawSpace};

pub(super) struct TypeSpecNonBridge {
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
}
