// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::AchiBaseAddr;
use crate::mem::accessor::Single;
use bitfield::bitfield;
use x86_64::PhysAddr;

pub struct Generic {
    pub cap: Single<HbaCapability>,
    pub ghc: Single<GlobalHbaControl>,
    pub pi: Single<PortsImplemented>,
    pub bohc: Single<BiosOsHandoffControlAndStatus>,
}
impl Generic {
    /// SAFETY: This method is unsafe because if `abar` is not the valid AHCI base address, this
    /// method can violate memory safety.
    pub unsafe fn new(abar: AchiBaseAddr) -> Self {
        let abar = PhysAddr::from(abar);

        let cap = crate::mem::accessor::user(abar).expect("Address is not aligned.");
        let ghc = crate::mem::accessor::user(abar + 0x04_usize).expect("Address is not aligned.");
        let pi = crate::mem::accessor::user(abar + 0x0c_usize).expect("Address is not aligned.");
        let bohc = crate::mem::accessor::user(abar + 0x28_usize).expect("Address is not aligned.");

        Self { cap, ghc, pi, bohc }
    }
}

bitfield! {
    #[repr(transparent)]
    #[derive(Copy,Clone)]
    pub struct HbaCapability(u32);
    impl Debug;
    pub num_of_command_slots, _: 12, 8;
    pub supports_64bit_addressing, _: 31;
}

bitfield! {
    #[repr(transparent)]
    #[derive(Copy,Clone)]
    pub struct GlobalHbaControl(u32);
    impl Debug;
    pub _, set_ahci_enable: 31;
    pub hba_reset, set_hba_reset: 0;
}

bitfield! {
    #[repr(transparent)]
    #[derive(Copy,Clone)]
    pub struct BiosOsHandoffControlAndStatus(u32);
    impl Debug;
    pub os_owned_semaphore, set_os_owned_semaphore: 1;
    pub bios_owned_semaphore, _: 0;
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct PortsImplemented(pub u32);
