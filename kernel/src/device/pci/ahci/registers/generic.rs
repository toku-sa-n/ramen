// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::AchiBaseAddr;
use crate::mem::accessor::Accessor;
use bitfield::bitfield;
use os_units::Bytes;

pub struct Generic {
    pub cap: Accessor<HbaCapability>,
    pub ghc: Accessor<GlobalHbaControl>,
    pub pi: Accessor<PortsImplemented>,
    pub bohc: Accessor<BiosOsHandoffControlAndStatus>,
}
impl Generic {
    /// SAFETY: This method is unsafe because if `abar` is not the valid AHCI base address, this
    /// method can violate memory safety.
    pub unsafe fn new(abar: AchiBaseAddr) -> Self {
        let cap = Accessor::new(abar.into(), Bytes::new(0x00));
        let ghc = Accessor::new(abar.into(), Bytes::new(0x04));
        let pi = Accessor::new(abar.into(), Bytes::new(0x0c));
        let bohc = Accessor::new(abar.into(), Bytes::new(0x28));

        Self { cap, ghc, pi, bohc }
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct HbaCapability(u32);
    impl Debug;
    pub num_of_command_slots, _: 12, 8;
    pub supports_64bit_addressing, _: 31;
}

bitfield! {
    #[repr(transparent)]
    pub struct GlobalHbaControl(u32);
    impl Debug;
    pub _, set_ahci_enable: 31;
    pub hba_reset, set_hba_reset: 0;
}

bitfield! {
    #[repr(transparent)]
    pub struct BiosOsHandoffControlAndStatus(u32);
    impl Debug;
    pub os_owned_semaphore, set_os_owned_semaphore: 1;
    pub bios_owned_semaphore, _: 0;
}

#[repr(transparent)]
#[derive(Debug)]
pub struct PortsImplemented(pub u32);
