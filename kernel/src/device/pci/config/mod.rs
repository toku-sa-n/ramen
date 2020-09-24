// SPDX-License-Identifier: GPL-3.0-or-later

pub mod bar;
mod common;
mod endpoint;
pub mod msi_x;

use {
    self::common::Common,
    bar::{Bar, BarIndex},
    core::ops::Add,
    endpoint::EndPoint,
    msi_x::MsiX,
    x86_64::instructions::port::{PortReadOnly, PortWriteOnly},
};

#[derive(Debug)]
pub struct Space {
    common: Common,
    endpoint: Option<EndPoint>,
    capability_ptr: Option<Offset>,
    msi_x: Option<MsiX>,
}

impl Space {
    pub fn fetch(bus: Bus, device: Device) -> Option<Self> {
        let common = Common::fetch(bus, device)?;

        let capability_ptr;
        let msi_x;

        if common.has_capability_ptr() {
            capability_ptr = Some(fetch_capability_ptr(bus, device));
            msi_x = if CapabilityId::new(bus, device, capability_ptr.unwrap()).is_msi_x() {
                Some(MsiX::new(bus, device, capability_ptr.unwrap()))
            } else {
                None
            };
        } else {
            capability_ptr = None;
            msi_x = None;
        };

        let endpoint = if common.is_endpoint() {
            Some(EndPoint::fetch(bus, device))
        } else {
            None
        };

        Some(Self {
            common,
            endpoint,
            capability_ptr,
            msi_x,
        })
    }

    pub fn is_xhci(&self) -> bool {
        self.common.is_xhci()
    }

    pub fn bar(&self) -> &Bar {
        &self.bar
    }
}

struct ConfigAddress {
    bus: Bus,
    device: Device,
    function: Function,
    register: Offset,
}

impl ConfigAddress {
    const PORT_CONFIG_ADDR: PortWriteOnly<u32> = PortWriteOnly::new(0xcf8);
    const PORT_CONFIG_DATA: PortReadOnly<u32> = PortReadOnly::new(0xcfc);

    #[allow(clippy::too_many_arguments)]
    fn new(bus: Bus, device: Device, function: Function, register: Offset) -> Self {
        Self {
            bus,
            device,
            function,
            register,
        }
    }

    fn as_u32(&self) -> u32 {
        const VALID: u32 = 0x8000_0000;
        let bus = self.bus.as_u32();
        let device = self.device.as_u32();
        let function = self.function.as_u32();
        let register = self.register.as_u32();

        VALID | bus << 16 | device << 11 | function << 8 | register
    }

    /// Safety: `self` must contain the valid config address.
    unsafe fn read(&self) -> u32 {
        Self::PORT_CONFIG_ADDR.write(self.as_u32());
        Self::PORT_CONFIG_DATA.read()
    }
}

fn fetch_capability_ptr(bus: Bus, device: Device) -> Offset {
    let config_addr = ConfigAddress::new(bus, device, Function::zero(), Offset::new(0x34));
    let raw_data = unsafe { config_addr.read() };

    Offset::new(raw_data & 0xfc)
}

#[derive(Debug, Copy, Clone)]
pub struct Bus(u32);
impl Bus {
    pub fn new(bus: u32) -> Self {
        Self(bus)
    }

    fn as_u32(self) -> u32 {
        self.0
    }
}

#[derive(Copy, Clone)]
pub struct Device(u32);
impl Device {
    pub fn new(device: u32) -> Self {
        assert!(device < 32);
        Self(device)
    }

    fn as_u32(self) -> u32 {
        self.0
    }
}

#[derive(Copy, Clone)]
pub struct Function(u32);
impl Function {
    pub fn new(function: u32) -> Self {
        assert!(function < 8);
        Self(function)
    }

    pub fn zero() -> Self {
        Self(0)
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Offset(u32);
impl Offset {
    pub fn new(offset: u32) -> Self {
        assert!(offset.trailing_zeros() >= 2);
        assert!(offset < 0x100);
        Self(offset)
    }

    fn zero() -> Self {
        Self(0)
    }

    fn as_u32(self) -> u32 {
        self.0
    }

    fn is_null(self) -> bool {
        self.0 == 0
    }
}

impl Add<u32> for Offset {
    type Output = Offset;

    fn add(self, rhs: u32) -> Self::Output {
        Self(self.0 + rhs)
    }
}

#[derive(Copy, Clone)]
struct CapabilityId(u32);
impl CapabilityId {
    fn new(bus: Bus, device: Device, capability_ptr: Offset) -> Self {
        let config_addr = ConfigAddress::new(bus, device, Function::zero(), capability_ptr);
        let raw = unsafe { config_addr.read() };

        Self(raw & 0xff)
    }

    fn is_msi_x(self) -> bool {
        self.0 == 0x11
    }
}
