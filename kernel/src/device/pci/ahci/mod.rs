// SPDX-License-Identifier: GPL-3.0-or-later

pub async fn task() {
    for device in super::iter_devices() {
        if device.is_ahci() {
            info!("SATA");
        }
    }
}
