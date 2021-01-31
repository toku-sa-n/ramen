// SPDX-License-Identifier: GPL-3.0-or-later

use page_box::PageBox;
use x86_64::PhysAddr;
use xhci::context::{byte32, byte64, DeviceHandler, InputControlHandler, InputHandler};

use super::registers;

pub(in crate::device::pci::xhci) struct Context {
    pub input: Input,
    pub output: PageBox<Device>,
}
impl Default for Context {
    fn default() -> Self {
        Self {
            input: Input::default(),
            output: Device::default().into(),
        }
    }
}

pub enum Input {
    Byte64(PageBox<byte64::Input>),
    Byte32(PageBox<byte32::Input>),
}
impl Input {
    pub fn control_mut(&mut self) -> &mut dyn InputControlHandler {
        match self {
            Self::Byte32(b32) => b32.control_mut(),
            Self::Byte64(b64) => b64.control_mut(),
        }
    }

    pub fn device_mut(&mut self) -> &mut dyn DeviceHandler {
        match self {
            Self::Byte32(b32) => b32.device_mut(),
            Self::Byte64(b64) => b64.device_mut(),
        }
    }

    pub fn phys_addr(&self) -> PhysAddr {
        match self {
            Self::Byte32(b32) => b32.phys_addr(),
            Self::Byte64(b64) => b64.phys_addr(),
        }
    }
}
impl Default for Input {
    fn default() -> Self {
        if csz() {
            Self::Byte64(byte64::Input::default().into())
        } else {
            Self::Byte32(byte32::Input::default().into())
        }
    }
}

pub(in crate::device::pci::xhci) enum Device {
    Byte64(PageBox<byte64::Device>),
    Byte32(PageBox<byte32::Device>),
}
impl Default for Device {
    fn default() -> Self {
        if csz() {
            Self::Byte64(byte64::Device::default().into())
        } else {
            Self::Byte32(byte32::Device::default().into())
        }
    }
}

fn csz() -> bool {
    registers::handle(|r| r.capability.hccparams1.read().context_size())
}
