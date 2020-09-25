// SPDX-License-Identifier: GPL-3.0-or-later

pub mod bar;
mod common;
pub mod msi_x;
pub mod type_spec;

use {
    self::common::Common,
    bar::Bar,
    core::ops::Add,
    type_spec::TypeSpec,
    x86_64::instructions::port::{PortReadOnly, PortWriteOnly},
};

const NUM_REGISTERS: usize = 16;

#[derive(Debug)]
pub struct Space {
    common: Common,
    type_spec: TypeSpec,
    capability_ptr: Option<Offset>,
}

impl Space {
    pub fn fetch(bus: Bus, device: Device) -> Option<Self> {
        let raw = RawSpace::fetch(bus, device)?;
        Some(Self::parse_raw(&raw))
    }

    fn parse_raw(raw: &RawSpace) -> Self {
        let common = Common::parse_raw(&raw);
        let type_spec = TypeSpec::parse_raw(&raw, &common);
        let capability_ptr = parse_raw_to_get_capability_ptr(&raw, &common);

        Self {
            common,
            type_spec,
            capability_ptr,
        }
    }

    pub fn is_xhci(&self) -> bool {
        self.common.is_xhci()
    }

    pub fn type_spec(&self) -> &TypeSpec {
        &self.type_spec
    }
}

struct RawSpace([u32; NUM_REGISTERS]);
impl RawSpace {
    fn fetch(bus: Bus, device: Device) -> Option<Self> {
        if !Self::valid(bus, device) {
            return None;
        }

        let mut raw = [0u32; NUM_REGISTERS];
        for i in (0..NUM_REGISTERS).step_by(4) {
            let config_addr =
                ConfigAddress::new(bus, device, Function::zero(), Offset::new(i as _));
            raw[i / 4] = unsafe { config_addr.read() };
        }

        Some(Self(raw))
    }

    fn valid(bus: Bus, device: Device) -> bool {
        let config_addr = ConfigAddress::new(bus, device, Function::zero(), Offset::zero());
        let id = unsafe { config_addr.read() };

        id != !0
    }

    fn as_slice(&self) -> &[u32] {
        &self.0
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
        let mut addr = Self::PORT_CONFIG_ADDR;
        addr.write(self.as_u32());

        let mut data = Self::PORT_CONFIG_DATA;
        data.read()
    }
}

fn parse_raw_to_get_capability_ptr(raw: &RawSpace, common: &Common) -> Option<Offset> {
    if common.has_capability_ptr() {
        Some(Offset::new(raw.as_slice()[0x0d] & 0xfc))
    } else {
        None
    }
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
