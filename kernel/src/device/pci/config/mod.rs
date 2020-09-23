// SPDX-License-Identifier: GPL-3.0-or-later

pub mod bar;
pub mod msi_x;

use {
    bar::Bar,
    core::convert::TryFrom,
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
    function: u8,
    register: u8,
}

impl ConfigAddress {
    const PORT_CONFIG_ADDR: u16 = 0xcf8;
    const PORT_CONFIG_DATA: u16 = 0xcfc;

    #[allow(clippy::too_many_arguments)]
    fn new(bus: Bus, device: Device, function: u8, register: u8) -> Self {
        assert!(function < 8);
        assert!(register.trailing_zeros() >= 2);

        Self {
            bus,
            device,
            function,
            register,
        }
    }

    fn as_u32(&self) -> u32 {
        const VALID: u32 = 0x8000_0000;
        let bus = u32::from(self.bus.as_u8());
        let device = u32::from(self.device.as_u8());
        let function = u32::from(self.function);
        let register = u32::from(self.register);

        VALID | bus << 16 | device << 11 | function << 8 | register
    }

    /// Safety: `self` must contain the valid config address.
    unsafe fn read(&self) -> u32 {
        PortWriteOnly::new(Self::PORT_CONFIG_ADDR).write(self.as_u32());
        PortReadOnly::new(Self::PORT_CONFIG_DATA).read()
    }
}

#[derive(Debug)]
struct Id {
    vendor: u16,
    device: u16,
}

impl Id {
    fn fetch(bus: Bus, device: Device) -> Self {
        let config_addr = ConfigAddress::new(bus, device, 0, 0);
        let raw_ids = unsafe { config_addr.read() };
        Self {
            vendor: u16::try_from(raw_ids & 0xffff).unwrap(),
            device: u16::try_from(raw_ids >> 16).unwrap(),
        }
    }

    fn is_valid(&self) -> bool {
        self.vendor != 0xffff
    }
}

#[derive(Debug)]
struct Class {
    base: u8,
    sub: u8,
}

impl Class {
    fn fetch(bus: Bus, device: Device) -> Self {
        let config_addr = ConfigAddress::new(bus, device, 0, 8);
        let raw_data = unsafe { config_addr.read() };

        Self {
            base: u8::try_from((raw_data >> 24) & 0xff).unwrap(),
            sub: u8::try_from((raw_data >> 16) & 0xff).unwrap(),
        }
    }
}

#[derive(Debug)]
struct Interface(u8);

impl Interface {
    fn fetch(bus: Bus, device: Device) -> Self {
        let config_addr = ConfigAddress::new(bus, device, 0, 8);
        let raw_data = unsafe { config_addr.read() };

        Self(u8::try_from((raw_data >> 8) & 0xff).unwrap())
    }
}

#[derive(Debug)]
struct CapabilityPtr(u8);

impl CapabilityPtr {
    fn fetch(bus: Bus, device: Device) -> Self {
        let config_addr = ConfigAddress::new(bus, device, 0, 0x34);
        let raw_data = unsafe { config_addr.read() };

        Self(u8::try_from(raw_data & 0xff).unwrap())
    }

    fn as_u8(&self) -> u8 {
        self.0
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Bus(u8);
impl Bus {
    pub fn new(bus: u8) -> Self {
        Self(bus)
    }

    fn as_u8(&self) -> u8 {
        self.0
    }
}

#[derive(Copy, Clone)]
pub struct Device(u8);
impl Device {
    pub fn new(device: u8) -> Self {
        assert!(device < 32);
        Self(device)
    }

    fn as_u8(&self) -> u8 {
        self.0
    }
}
