// SPDX-License-Identifier: GPL-3.0-or-later

use super::{Bar, BarIndex, Bus, Device};

pub(super) struct EndPoint {
    bar: [Bar; 6],
}

impl EndPoint {
    pub(super) fn fetch(bus: Bus, device: Device) -> Self {
        let bar: [Bar; 6];
        for i in 0..6u32 {
            bar[i as usize] = Bar::fetch(bus, device, BarIndex::new(i));
        }

        Self { bar }
    }
}
