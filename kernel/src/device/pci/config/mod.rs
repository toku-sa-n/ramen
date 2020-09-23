// SPDX-License-Identifier: GPL-3.0-or-later

pub mod bar;
pub mod msi_x;

use {
    bar::Bar,
    core::ops::Add,
    x86_64::instructions::port::{PortReadOnly, PortWriteOnly},
};

#[derive(Debug)]
pub struct Space {
    id: Id,
    bar: Bar,
    class: Class,
    interface: Interface,
    capability_ptr: CapabilityPtr,
}

impl Space {
    pub fn fetch(bus: Bus, device: Device) -> Option<Self> {
        let id = Id::fetch(bus, device);
        if !id.is_valid() {
            return None;
        }

        let bar = Bar::fetch(bus, device);
        let class = Class::fetch(bus, device);
        let interface = Interface::fetch(bus, device);
        let capability_ptr = CapabilityPtr::fetch(bus, device);

        Some(Self {
            id,
            bar,
            class,
            interface,
            capability_ptr,
        })
    }

    pub fn is_xhci(&self) -> bool {
        self.class.base == 0x0c && self.class.sub == 0x03 && self.interface.0 == 0x30
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

#[derive(Debug)]
struct Id {
    vendor: u32,
    device: u32,
}

impl Id {
    fn fetch(bus: Bus, device: Device) -> Self {
        let config_addr = ConfigAddress::new(bus, device, Function::zero(), Offset::zero());
        let raw_ids = unsafe { config_addr.read() };
        Self {
            vendor: raw_ids & 0xffff,
            device: raw_ids >> 16,
        }
    }

    fn is_valid(&self) -> bool {
        self.vendor != 0xffff
    }
}

#[derive(Debug)]
struct Class {
    base: u32,
    sub: u32,
}

impl Class {
    fn fetch(bus: Bus, device: Device) -> Self {
        let config_addr = ConfigAddress::new(bus, device, Function::zero(), Offset::new(8));
        let raw_data = unsafe { config_addr.read() };

        Self {
            base: (raw_data >> 24) & 0xff,
            sub: (raw_data >> 16) & 0xff,
        }
    }
}

#[derive(Debug)]
struct Interface(u32);

impl Interface {
    fn fetch(bus: Bus, device: Device) -> Self {
        let config_addr = ConfigAddress::new(bus, device, Function::zero(), Offset::new(8));
        let raw_data = unsafe { config_addr.read() };

        Self((raw_data >> 8) & 0xff)
    }
}

#[derive(Debug)]
struct CapabilityPtr(u32);

impl CapabilityPtr {
    fn fetch(bus: Bus, device: Device) -> Self {
        let config_addr = ConfigAddress::new(bus, device, Function::zero(), Offset::new(0x34));
        let raw_data = unsafe { config_addr.read() };

        Self(raw_data & 0xff)
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

#[derive(Copy, Clone)]
pub struct Offset(u32);
impl Offset {
    pub fn new(offset: u32) -> Self {
        assert!(offset.trailing_zeros() >= 2);
        Self(offset)
    }

    fn zero() -> Self {
        Self(0)
    }

    fn as_u32(self) -> u32 {
        self.0
    }
}

impl Add<u32> for Offset {
    type Output = Offset;

    fn add(self, rhs: u32) -> Self::Output {
        Self(self.0 + rhs)
    }
}
