// SPDX-License-Identifier: GPL-3.0-or-later

mod ahc;
mod port;
mod registers;

use {
    crate::device::pci::{self, config::bar},
    ahc::Ahc,
    alloc::rc::Rc,
    core::cell::RefCell,
    registers::Registers,
    x86_64::PhysAddr,
};

pub async fn task() {
    let (mut ahc, mut ports) = match init() {
        Some(x) => x,
        None => return,
    };

    place_into_minimally_initialized_state(&mut ahc, &mut ports);
    ahc.get_ownership_from_bios();
}

fn init() -> Option<(Ahc, port::Collection)> {
    let registers = Rc::new(RefCell::new(fetch_registers()?));
    let ahc = Ahc::new(registers.clone());
    let port_collection = port::Collection::new(&registers);

    Some((ahc, port_collection))
}

fn fetch_registers() -> Option<Registers> {
    let abar = AchiBaseAddr::new()?;
    Some(Registers::new(abar))
}

fn place_into_minimally_initialized_state(ahc: &mut Ahc, ports: &mut port::Collection) {
    ahc.reset();
    ahc.indicate_system_software_is_ahci_aware();
    ports.idle_all_ports();
    ports.register_command_lists_and_fis();
    ports.clear_error_bits();
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
