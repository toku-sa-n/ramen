// SPDX-License-Identifier: GPL-3.0-or-later

pub mod config;

use config::{Bus, Device};

pub fn iter_devices() -> impl Iterator<Item = config::Space> {
    IterPciDevices::new(0, 0)
}

struct IterPciDevices {
    bus: u8,
    device: u8,
}

impl IterPciDevices {
    fn new(bus: u8, device: u8) -> Self {
        assert!(device < 32);
        Self { bus, device }
    }
}

impl Iterator for IterPciDevices {
    type Item = config::Space;

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
