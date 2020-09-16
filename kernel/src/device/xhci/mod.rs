// SPDX-License-Identifier: GPL-3.0-or-later

mod capability_register;
mod usb_leg_sup;

use super::pci::config;
use capability_register::CapabilityRegister;

pub struct Xhci {
    config_space: config::Space,
    capability_register: CapabilityRegister,
}

impl Xhci {
    fn new(config_space: config::Space) -> Result<Self, Error> {
        if config_space.is_xhci() {
            let capability_register = CapabilityRegister::fetch(&config_space.bar());
            Ok(Self {
                config_space,
                capability_register,
            })
        } else {
            Err(Error::NotXhciDevice)
        }
    }

    fn get_ownership_from_bios(&self) {
        info!("Getting ownership from BIOS...");
        todo!();
        info!("Done");
    }
}
#[derive(Debug)]
enum Error {
    NotXhciDevice,
}

pub fn iter_devices() -> impl Iterator<Item = Xhci> {
    super::pci::iter_devices().filter_map(|device| {
        if device.is_xhci() {
            Xhci::new(device).ok()
        } else {
            None
        }
    })
}
