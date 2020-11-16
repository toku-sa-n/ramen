// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{generic::Generic, AchiBaseAddr},
    crate::mem::accessor::Accessor,
    bitfield::bitfield,
    os_units::Bytes,
    x86_64::PhysAddr,
};

pub struct Registers {
    pub clb: Accessor<PortxCommandListBaseAddress>,
    pub fb: Accessor<PortxFisBaseAddress>,
    pub cmd: Accessor<PortxCommandAndStatus>,
    pub tfd: Accessor<PortxTaskFileData>,
    pub sig: Accessor<PortxSignature>,
    pub ssts: Accessor<PortxSerialAtaStatus>,
    pub serr: Accessor<PortxSerialAtaError>,
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

        let clb = Accessor::new(base_addr, Bytes::new(0x00));
        let fb = Accessor::new(base_addr, Bytes::new(0x08));
        let cmd = Accessor::new(base_addr, Bytes::new(0x18));
        let tfd = Accessor::new(base_addr, Bytes::new(0x20));
        let sig = Accessor::new(base_addr, Bytes::new(0x24));
        let ssts = Accessor::new(base_addr, Bytes::new(0x28));
        let serr = Accessor::new(base_addr, Bytes::new(0x30));

        Self {
            clb,
            fb,
            cmd,
            tfd,
            sig,
            ssts,
            serr,
        }
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

#[repr(transparent)]
pub struct PortxSerialAtaError(pub u32);

#[repr(transparent)]
pub struct PortxCommandListBaseAddress(u64);
impl PortxCommandListBaseAddress {
    pub fn set(&mut self, addr: PhysAddr) {
        assert!(addr.as_u64().trailing_zeros() >= 10);
        self.0 = addr.as_u64();
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct PortxTaskFileData(u32);
    impl Debug;
    pub busy, _: 14;
    pub data_transfer_is_required, _: 10;
}

#[repr(transparent)]
pub struct PortxSignature(u32);
impl PortxSignature {
    pub fn get(&self) -> u32 {
        self.0
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct PortxSerialAtaStatus(u32);
    impl Debug;
    pub device_detection, _: 3, 0;
    pub interface_power_management, _: 11, 8;
}

#[repr(transparent)]
pub struct PortxFisBaseAddress(u64);
impl PortxFisBaseAddress {
    pub fn set(&mut self, addr: PhysAddr) {
        assert!(addr.as_u64().trailing_zeros() >= 8);
        self.0 = addr.as_u64();
    }
}
