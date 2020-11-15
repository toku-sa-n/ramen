// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::super::AchiBaseAddr, crate::mem::accessor::Accessor, bitfield::bitfield, os_units::Bytes,
};

pub struct Generic {
    pub ghc: Accessor<GlobalHbaControl>,
    pub bohc: Accessor<BiosOsHandoffControlAndStatus>,
}
impl Generic {
    pub fn new(abar: AchiBaseAddr) -> Self {
        let ghc = Accessor::new(abar.into(), Bytes::new(0x04));
        let bohc = Accessor::new(abar.into(), Bytes::new(0x28));

        Self { ghc, bohc }
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct GlobalHbaControl(u32);
    impl Debug;
    pub _, set_ahci_enable: 31;
}

bitfield! {
    #[repr(transparent)]
    pub struct BiosOsHandoffControlAndStatus(u32);
    impl Debug;
    pub os_owned_semaphore, set_os_owned_semaphore: 1;
    pub bios_owned_semaphore, _: 0;
}
