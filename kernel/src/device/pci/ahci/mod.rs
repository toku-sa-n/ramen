// SPDX-License-Identifier: GPL-3.0-or-later

mod ahc;
mod registers;

use {
    crate::device::pci::{self, config::bar},
    ahc::Ahc,
    x86_64::PhysAddr,
};

pub async fn task() {
    let mut ahc = match Ahc::new() {
        Some(ahc) => ahc,
        None => return,
    };
    ahc.place_into_minimally_initialized_state();
    ahc.get_ownership_from_bios();
}

#[derive(Copy, Clone)]
pub struct AchiBaseAddr(PhysAddr);
impl AchiBaseAddr {
    fn new() -> Option<Self> {
        for device in pci::iter_devices() {
            if device.is_ahci() {
                let addr = device.base_address(bar::Index::new(5));
                return Some(Self(addr));
            }
        }

        None
    }
}
impl From<AchiBaseAddr> for PhysAddr {
    fn from(abar: AchiBaseAddr) -> Self {
        abar.0
    }
}
