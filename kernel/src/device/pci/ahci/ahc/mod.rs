// SPDX-License-Identifier: GPL-3.0-or-later

mod registers;

use {
    crate::device::pci::{self, config::bar},
    registers::Registers,
    x86_64::PhysAddr,
};

pub struct Ahc {
    registers: Registers,
}
impl Ahc {
    pub fn new() -> Option<Self> {
        let registers = Self::fetch_registers()?;
        Some(Self { registers })
    }

    pub fn get_ownership_from_bios(&mut self) {
        self.request_ownership_to_bios();
        self.wait_until_ownership_is_moved();
    }

    fn request_ownership_to_bios(&mut self) {
        let bohc = &mut self.registers.generic.bohc;
        bohc.update(|bohc| bohc.set_os_owned_semaphore(true));
    }

    fn wait_until_ownership_is_moved(&self) {
        let bohc = &self.registers.generic.bohc;
        while bohc.read().os_owned_semaphore() && !bohc.read().bios_owned_semaphore() {}
    }

    fn fetch_registers() -> Option<Registers> {
        let abar = AchiBaseAddr::new()?;
        Some(Registers::new(abar))
    }
}

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
