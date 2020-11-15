// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{generic::Generic, AchiBaseAddr},
    crate::mem::accessor::Accessor,
    bitfield::bitfield,
    os_units::Bytes,
    x86_64::PhysAddr,
};

pub struct Registers {
    pub px_cmd: Accessor<PortxCommandAndStatus>,
}
impl Registers {
    pub fn new(abar: AchiBaseAddr, port_index: usize, generic: &Generic) -> Option<Self> {
        if Self::exist(port_index, generic) {
            Some(Self::fetch(abar, port_index))
        } else {
            None
        }
    }

    fn exist(port_index: usize, generic: &Generic) -> bool {
        generic.pi.read().0 & (1 << port_index) != 0
    }

    fn fetch(abar: AchiBaseAddr, port_index: usize) -> Self {
        let base_addr = Self::base_addr_to_registers(abar, port_index);
        let px_cmd = Accessor::new(base_addr, Bytes::new(0x18));

        Self { px_cmd }
    }

    fn base_addr_to_registers(abar: AchiBaseAddr, port_index: usize) -> PhysAddr {
        PhysAddr::from(abar) + 0x100_usize + port_index * 0x80
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct PortxCommandAndStatus(u32);
    impl Debug;
    pub start_bit, set_start_bit: 0;
    pub fis_receive_enable, set_fis_receive_enable: 4;
    pub fis_receive_running, _: 14;
    pub command_list_running, _: 15;
}
