// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;

use acpi::AcpiTables;
use x86_64::{instructions::port::PortReadOnly, PhysAddr};

use crate::mem::allocator;

pub struct AcpiPmTimer {
    reg: PortReadOnly<u32>,
    supported: SupportedBits,
}
impl AcpiPmTimer {
    /// Safety: This method is unsafe because `rsdp` must be a valid RSDP. Otherwise it may break
    /// memory safety by dereferencing to an invalid address.
    pub unsafe fn new(rsdp: PhysAddr) -> Self {
        let table = Self::fetch_apic(rsdp);
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

    /// Safety: This method is unsafe because `rsdp` must be a valid RSDP. Otherwise it may break
    /// memory safety by dereferencing to an invalid address.
    unsafe fn fetch_apic(rsdp: PhysAddr) -> AcpiTables<allocator::acpi::Mapper> {
        let mapper = allocator::acpi::Mapper;
        AcpiTables::from_rsdp(mapper, rsdp.as_u64().try_into().unwrap()).unwrap()
    }

    pub fn wait_milliseconds(&mut self, t: u32) {
        const FREQUENCY: u32 = 3579545;
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
