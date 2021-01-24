// SPDX-License-Identifier: GPL-3.0-or-later

use super::capability::Capability;
use crate::mem::accessor::{Array, Single};
use x86_64::PhysAddr;
use xhci::registers::operational::{
    CommandRingControlRegister, ConfigureRegister, DeviceContextBaseAddressArrayPointerRegister,
    PageSizeRegister, PortStatusAndControlRegister, UsbCommandRegister, UsbStatusRegister,
};

pub struct Operational {
    pub usb_cmd: Single<UsbCommandRegister>,
    pub usb_sts: Single<UsbStatusRegister>,
    pub page_size: Single<PageSizeRegister>,
    pub crcr: Single<CommandRingControlRegister>,
    pub dcbaap: Single<DeviceContextBaseAddressArrayPointerRegister>,
    pub config: Single<ConfigureRegister>,
    pub port_registers: Array<PortRegisters>,
}

impl Operational {
    /// SAFETY: This method is unsafe because if `mmio_base` is not a valid MMIO base address, it
    /// can violate memory safety.
    #[allow(clippy::too_many_lines)]
    pub unsafe fn new(mmio_base: PhysAddr, capabilities: &Capability) -> Self {
        let operational_base = mmio_base + u64::from(capabilities.cap_length.read().get());

        macro_rules! accessor {
            ($bytes:expr) => {
                crate::mem::accessor::user(operational_base + $bytes as usize)
                    .expect("Address is not aligned.")
            };
        }

        let usb_cmd = accessor!(0x00);
        let usb_sts = accessor!(0x04);
        let page_size = accessor!(0x08);
        let crcr = accessor!(0x18);
        let dcbaap = accessor!(0x30);
        let config = accessor!(0x38);
        let port_registers = crate::mem::accessor::user_array(
            operational_base + 0x400_usize,
            capabilities.hcs_params_1.read().number_of_ports().into(),
        )
        .expect("Address is not aligned.");

        Self {
            usb_cmd,
            usb_sts,
            page_size,
            crcr,
            dcbaap,
            config,
            port_registers,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PortRegisters {
    pub port_sc: PortStatusAndControlRegister,
    _port_pmsc: u32,
    _port_li: u32,
    _port_hlpmc: u32,
}
