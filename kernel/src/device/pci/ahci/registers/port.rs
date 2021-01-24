// SPDX-License-Identifier: GPL-3.0-or-later

use super::{generic::Generic, AchiBaseAddr};
use crate::mem::accessor::Single;
use bitfield::bitfield;
use x86_64::PhysAddr;

pub struct Registers {
    pub clb: Single<PortxCommandListBaseAddress>,
    pub fb: Single<PortxFisBaseAddress>,
    pub cmd: Single<PortxCommandAndStatus>,
    pub tfd: Single<PortxTaskFileData>,
    pub sig: Single<PortxSignature>,
    pub ssts: Single<PortxSerialAtaStatus>,
    pub serr: Single<PortxSerialAtaError>,
    pub sact: Single<PortxSerialAtaActive>,
    pub ci: Single<PortxCommandIssue>,
}
impl Registers {
    /// SAFETY: This method is unsafe because if `abar` is not a valid AHCI base address, it can
    /// violate memory safety.
    pub unsafe fn new(abar: AchiBaseAddr, port_index: usize, generic: &Generic) -> Option<Self> {
        if Self::exist(port_index, generic) {
            Some(Self::fetch(abar, port_index))
        } else {
            None
        }
    }

    fn exist(port_index: usize, generic: &Generic) -> bool {
        generic.pi.read().0 & (1 << port_index) != 0
    }

    /// SAFETY: This method is unsafe because if `abar` is not a valid AHCI base address, it can
    /// violate memory safety.
    unsafe fn fetch(abar: AchiBaseAddr, port_index: usize) -> Self {
        let base_addr = Self::base_addr_to_registers(abar, port_index);

        macro_rules! new_accessor {
            ($offset:expr) => {
                crate::mem::accessor::user(base_addr + $offset as usize)
                    .expect("Address is not aligned.")
            };
        }

        Self {
            clb: new_accessor!(0x00),
            fb: new_accessor!(0x08),
            cmd: new_accessor!(0x18),
            tfd: new_accessor!(0x20),
            sig: new_accessor!(0x24),
            ssts: new_accessor!(0x28),
            serr: new_accessor!(0x30),
            sact: new_accessor!(0x34),
            ci: new_accessor!(0x38),
        }
    }

    fn base_addr_to_registers(abar: AchiBaseAddr, port_index: usize) -> PhysAddr {
        PhysAddr::from(abar) + 0x100_usize + port_index * 0x80
    }
}

bitfield! {
    #[repr(transparent)]
    #[derive(Copy,Clone)]
    pub struct PortxCommandAndStatus(u32);
    impl Debug;
    pub start_bit, set_start_bit: 0;
    pub fis_receive_enable, set_fis_receive_enable: 4;
    pub fis_receive_running, _: 14;
    pub command_list_running, _: 15;
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PortxSerialAtaError(pub u32);

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PortxCommandListBaseAddress(u64);
impl PortxCommandListBaseAddress {
    pub fn set(&mut self, addr: PhysAddr) {
        assert!(addr.as_u64().trailing_zeros() >= 10);
        self.0 = addr.as_u64();
    }
}

bitfield! {
    #[repr(transparent)]
    #[derive(Copy,Clone)]
    pub struct PortxTaskFileData(u32);
    impl Debug;
    pub busy, _: 14;
    pub data_transfer_is_required, _: 10;
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PortxSignature(u32);
impl PortxSignature {
    pub fn get(self) -> u32 {
        self.0
    }
}

bitfield! {
    #[repr(transparent)]
    #[derive(Copy,Clone)]
    pub struct PortxSerialAtaStatus(u32);
    impl Debug;
    pub device_detection, _: 3, 0;
    pub interface_power_management, _: 11, 8;
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PortxFisBaseAddress(u64);
impl PortxFisBaseAddress {
    pub fn set(&mut self, addr: PhysAddr) {
        assert!(addr.as_u64().trailing_zeros() >= 8);
        self.0 = addr.as_u64();
    }
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PortxSerialAtaActive(u32);
impl PortxSerialAtaActive {
    pub fn get(self) -> u32 {
        self.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PortxCommandIssue(u32);
impl PortxCommandIssue {
    pub fn get(self) -> u32 {
        self.0
    }
}
