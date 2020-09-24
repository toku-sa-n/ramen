// SPDX-License-Identifier: GPL-3.0-or-later

pub mod config;

use config::{Bus, Device};

pub fn iter_devices() -> impl Iterator<Item = config::Space<'static>> {
    IterPciDevices::new(0, 0)
}

struct IterPciDevices {
    bus: u32,
    device: u32,
}

impl IterPciDevices {
    fn new(bus: u32, device: u32) -> Self {
        assert!(device < 32);
        Self { bus, device }
    }
}

impl Iterator for IterPciDevices {
    type Item = config::Space<'static>;

    fn next(&mut self) -> Option<Self::Item> {
        for bus in self.bus..=255 {
            for device in self.device..32 {
                if let Some(space) = config::Space::fetch(Bus::new(bus), Device::new(device)) {
                    self.bus = bus;
                    self.device = device + 1;

                    return Some(space);
                }
            }
        }

        None
    }
}
