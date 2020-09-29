// SPDX-License-Identifier: GPL-3.0-or-later

pub mod bar;
mod common;
mod extended_capability;
pub mod type_spec;

use {
    self::common::Common,
    alloc::boxed::Box,
    bar::Bar,
    core::{
        convert::{From, TryFrom},
        iter,
        ops::Add,
    },
    extended_capability::ExtendedCapability,
    type_spec::TypeSpec,
    x86_64::instructions::port::{PortReadOnly, PortWriteOnly},
};

const NUM_REGISTERS: usize = 64;

#[derive(Debug)]
pub struct Space {
    registers: Registers,
}

impl Space {
    pub fn new(bus: Bus, device: Device) -> Option<Self> {
        Some(Self {
            registers: Registers::new(bus, device)?,
        })
    }

    pub fn is_xhci(&self) -> bool {
        self.common().is_xhci()
    }

    pub fn type_spec(&self) -> TypeSpec {
        TypeSpec::new(&self.registers, &self.common())
    }

    pub fn iter_capability_registers<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = ExtendedCapability> + 'a> {
        let capability_pointer = CapabilityPointer::new(&self.registers, &self.common());
        match capability_pointer {
            None => Box::new(iter::empty()),
            Some(capability_pointer) => Box::new(extended_capability::Iter::new(
                &self.registers,
                capability_pointer.as_register_index(),
            )),
        }
    }

    fn common(&self) -> Common {
        Common::new(&self.registers)
    }

    fn capability_pointer_exists(&self) -> bool {
        self.common().has_capability_ptr()
    }
}

#[derive(Debug)]
pub struct Registers {
    bus: Bus,
    device: Device,
}
impl Registers {
    fn new(bus: Bus, device: Device) -> Option<Self> {
        if Self::valid(bus, device) {
            Some(Self { bus, device })
        } else {
            None
        }
    }

    fn valid(bus: Bus, device: Device) -> bool {
        let config_addr = ConfigAddress::new(bus, device, Function::zero(), RegisterIndex::zero());
        let id = unsafe { config_addr.read() };

        id != !0
    }

    fn get(&self, index: RegisterIndex) -> u32 {
        let accessor = ConfigAddress::new(self.bus, self.device, Function::zero(), index);
        unsafe { accessor.read() }
    }
}

struct ConfigAddress {
    bus: Bus,
    device: Device,
    function: Function,
    register: RegisterIndex,
}

impl ConfigAddress {
    const PORT_CONFIG_ADDR: PortWriteOnly<u32> = PortWriteOnly::new(0xcf8);
    const PORT_CONFIG_DATA: PortReadOnly<u32> = PortReadOnly::new(0xcfc);

    #[allow(clippy::too_many_arguments)]
    fn new(bus: Bus, device: Device, function: Function, register: RegisterIndex) -> Self {
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
        let register = u32::try_from(self.register.as_usize()).unwrap();

        VALID | bus << 16 | device << 11 | function << 8 | register << 2
    }

    /// Safety: `self` must contain the valid config address.
    unsafe fn read(&self) -> u32 {
        let mut addr = Self::PORT_CONFIG_ADDR;
        addr.write(self.as_u32());

        let mut data = Self::PORT_CONFIG_DATA;
        data.read()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Bus(u32);
impl Bus {
    pub fn new(bus: u32) -> Self {
        Self(bus)
    }

    fn as_u32(self) -> u32 {
        self.0
    }
}

#[derive(Copy, Clone, Debug)]
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
pub struct RegisterIndex(usize);
impl RegisterIndex {
    pub fn new(offset: usize) -> Self {
        assert!(offset < NUM_REGISTERS);
        Self(offset)
    }

    fn zero() -> Self {
        Self(0)
    }

    fn as_usize(self) -> usize {
        self.0
    }

    fn is_null(self) -> bool {
        self.0 == 0
    }
}

impl Add<usize> for RegisterIndex {
    type Output = RegisterIndex;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct CapabilityPointer<'a> {
    registers: &'a Registers,
}
impl<'a> CapabilityPointer<'a> {
    pub fn new(registers: &'a Registers, common: &Common) -> Option<Self> {
        if common.has_capability_ptr() {
            Some(Self { registers })
        } else {
            None
        }
    }

    pub fn as_register_index(self) -> RegisterIndex {
        let pointer = usize::try_from(self.registers.get(RegisterIndex::new(0x0d)) & 0xff).unwrap();
        RegisterIndex::new(pointer >> 2)
    }
}

#[derive(Copy, Clone)]
struct CapabilityId(u32);
impl CapabilityId {
    fn new(bus: Bus, device: Device, capability_ptr: RegisterIndex) -> Self {
        let config_addr = ConfigAddress::new(bus, device, Function::zero(), capability_ptr);
        let raw = unsafe { config_addr.read() };

        Self(raw & 0xff)
    }

    fn is_msi_x(self) -> bool {
        self.0 == 0x11
    }
}
