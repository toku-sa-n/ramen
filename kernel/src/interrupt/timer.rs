// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;

use acpi::AcpiTables;
use x86_64::instructions::port::PortReadOnly;

use crate::mem::allocator;

pub struct AcpiPmTimer {
    reg: PortReadOnly<u32>,
    supported: SupportedBits,
}
impl AcpiPmTimer {
    pub fn new(table: &AcpiTables<allocator::acpi::Mapper>) -> Self {
        let pm_timer = acpi::PmTimer::new(&table).unwrap();
        info!("Base: {}", pm_timer.io_base);
        Self {
            reg: PortReadOnly::new(pm_timer.io_base.try_into().unwrap()),
            supported: if pm_timer.supports_32bit {
                SupportedBits::Bits32
            } else {
                SupportedBits::Bits24
            },
        }
    }

    pub fn wait_milliseconds(&mut self, t: u32) {
        const FREQUENCY: u32 = 3_579_545;
        let start = unsafe { self.reg.read() };
        let mut end = start.wrapping_add(FREQUENCY.wrapping_mul(t / 1000));
        if let SupportedBits::Bits24 = self.supported {
            end &= 0x00ff_ffff;
        }

        if end < start {
            while unsafe { self.reg.read() >= start } {}
        }

        while unsafe { self.reg.read() < end } {}
    }
}

#[derive(Debug)]
enum SupportedBits {
    Bits32,
    Bits24,
}
