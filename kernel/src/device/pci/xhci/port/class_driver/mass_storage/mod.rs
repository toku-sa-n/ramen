// SPDX-License-Identifier: GPL-3.0-or-later

mod scsi;

use crate::device::pci::xhci::port::endpoint;

pub async fn task(eps: endpoint::Collection) {
    let m = MassStorage::new(eps);
    info!("This is the task of USB Mass Storage.");
}

struct MassStorage {
    eps: endpoint::Collection,
}
impl MassStorage {
    fn new(eps: endpoint::Collection) -> Self {
        Self { eps }
    }
}
