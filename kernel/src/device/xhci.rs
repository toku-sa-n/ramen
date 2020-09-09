// SPDX-License-Identifier: GPL-3.0-or-later

use super::pci::config;

pub fn iter_devices() -> impl Iterator<Item = config::Space> {
    super::pci::iter_devices().filter(|device| device.is_xhci())
}
