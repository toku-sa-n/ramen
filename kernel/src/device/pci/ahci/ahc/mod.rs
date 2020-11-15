// SPDX-License-Identifier: GPL-3.0-or-later

mod registers;

use {
    crate::device::pci::{self, config::bar},
    alloc::vec::Vec,
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

    pub fn place_into_minimally_initialized_state(&mut self) {
        self.indicate_system_software_is_ahci_aware();
        self.idle_ports();
    }

    pub fn get_ownership_from_bios(&mut self) {
        self.request_ownership_to_bios();
        self.wait_until_ownership_is_moved();
    }

    fn indicate_system_software_is_ahci_aware(&mut self) {
        let ghc = &mut self.registers.generic.ghc;
        ghc.update(|ghc| ghc.set_ahci_enable(true));
    }

    fn idle_ports(&mut self) {
        let available_ports = self.available_ports();
        for port_index in available_ports {
            if !self.port_is_idle(port_index) {
                self.make_port_idle(port_index)
            }
        }
    }

    fn available_ports(&self) -> Vec<usize> {
        (0..32).filter(|x| self.port_available(*x)).collect()
    }

    fn port_is_idle(&self, index: usize) -> bool {
        let px_cmd = &self.registers.port_regs[index].as_ref().unwrap().px_cmd;
        let px_cmd = px_cmd.read();
        !px_cmd.start_bit()
            & !px_cmd.command_list_running()
            & !px_cmd.fis_receive_enable()
            & !px_cmd.fis_receive_running()
    }

    fn make_port_idle(&mut self, index: usize) {
        let px_cmd = &mut self.registers.port_regs[index].as_mut().unwrap().px_cmd;

        px_cmd.update(|cmd| cmd.set_start_bit(false));
        while px_cmd.read().command_list_running() {}

        px_cmd.update(|cmd| cmd.set_fis_receive_enable(false));
        while px_cmd.read().fis_receive_running() {}
    }

    fn port_available(&self, port_index: usize) -> bool {
        assert!(port_index < 32);
        let pi = &self.registers.generic.pi;
        pi.read().0 & (1 << port_index) != 0
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
