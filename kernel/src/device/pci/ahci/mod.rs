// SPDX-License-Identifier: GPL-3.0-or-later

mod ahc;
mod port;
mod registers;

use crate::{
    device::pci::{self, config::bar},
    multitask::task,
};
use ahc::Ahc;
use alloc::rc::Rc;
use core::cell::RefCell;
use registers::Registers;
use x86_64::PhysAddr;

pub async fn task(task_collection: Rc<RefCell<task::Collection>>) {
    let (registers, mut ahc) = match init() {
        Some(x) => x,
        None => return,
    };

    ahc.init();
    port::spawn_tasks(&registers, &task_collection);
}

fn init() -> Option<(Rc<RefCell<Registers>>, Ahc)> {
    let registers = Rc::new(RefCell::new(fetch_registers()?));
    let ahc = Ahc::new(registers.clone());

    Some((registers, ahc))
}

fn fetch_registers() -> Option<Registers> {
    let abar = AchiBaseAddr::new()?;

    // Safety: This operation is safe because `abar` is generated from the 5th BAR.
    Some(unsafe { Registers::new(abar) })
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
