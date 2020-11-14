// SPDX-License-Identifier: GPL-3.0-or-later

use {super::config::bar, x86_64::PhysAddr};

pub async fn task() {
    let abar = match AchiBaseAddr::new() {
        Some(abar) => abar,
        None => return,
    };
}

struct AchiBaseAddr(PhysAddr);
impl AchiBaseAddr {
    fn new() -> Option<Self> {
        for device in super::iter_devices() {
            if device.is_ahci() {
                let addr = device.base_address(bar::Index::new(5));
                return Some(Self(addr));
            }
        }

        None
    }
}
