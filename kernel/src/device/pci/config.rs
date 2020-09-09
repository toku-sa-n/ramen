// SPDX-License-Identifier: GPL-3.0-or-later

mod bar;

use bar::Bar;
use core::convert::TryFrom;
use x86_64::instructions::port::{PortReadOnly, PortWriteOnly};

#[derive(Debug)]
pub struct Space {
    id: Id,
    bar: Bar,
}

impl Space {
    pub fn fetch(bus: u8, device: u8) -> Option<Self> {
        let id = Id::fetch(bus, device)?;
        let bar = Bar::fetch(bus, device);

        Some(Self { id, bar })
    }
}

struct ConfigAddress {
    bus: u8,
    device: u8,
    function: u8,
    register: u8,
}

impl ConfigAddress {
    const VALID: u32 = 0x8000_0000;
    const PORT_CONFIG_ADDR: u16 = 0xcf8;
    const PORT_CONFIG_DATA: u16 = 0xcfc;

    #[allow(clippy::too_many_arguments)]
    fn new(bus: u8, device: u8, function: u8, register: u8) -> Self {
        assert!(device < 32);
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
        let bus = u32::from(self.bus);
        let device = u32::from(self.device);
        let function = u32::from(self.function);
        let register = u32::from(self.register);

        Self::VALID | bus << 16 | device << 11 | function << 8 | register
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
    fn fetch(bus: u8, device: u8) -> Option<Self> {
        let config_addr = ConfigAddress::new(bus, device, 0, 0);
        let raw_ids = unsafe { config_addr.read() };
        if raw_ids & 0xffff == 0xffff {
            None
        } else {
            Some(Self {
                vendor: u16::try_from(raw_ids & 0xffff).unwrap(),
                device: u16::try_from(raw_ids >> 16).unwrap(),
            })
        }
    }
}
