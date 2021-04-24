// SPDX-License-Identifier: GPL-3.0-or-later

use crate::mem::{accessor::Single, allocator};
use acpi::{platform::address::AddressSpace, AcpiTables};
use core::convert::TryInto;
use log::info;
use x86_64::{instructions::port::PortReadOnly, PhysAddr};

const LVT_TIMER: PhysAddr = PhysAddr::new_truncate(0xfee0_0320);
const INITIAL_COUNT: PhysAddr = PhysAddr::new_truncate(0xfee0_0380);
const CURRENT_COUNT: PhysAddr = PhysAddr::new_truncate(0xfee0_0390);
const DIVIDE_CONFIG: PhysAddr = PhysAddr::new_truncate(0xfee0_03e0);
const TIMER_VECTOR: u8 = 0x20;

pub(crate) fn init(table: &AcpiTables<allocator::acpi::Mapper>) {
    let mut local_apic_tm = LocalApic::new(table);
    local_apic_tm.init();
}

struct LocalApic {
    lvt_timer: Single<u32>,
    initial_count: Single<u32>,
    current_count: Single<u32>,
    divide_config: Single<u32>,
    pm: AcpiPm,
    frequency: Option<u32>,
}
impl LocalApic {
    fn new(table: &AcpiTables<allocator::acpi::Mapper>) -> Self {
        // SAFETY: These operations are safe because the addresses are the correct ones.
        let lvt_timer = unsafe { crate::mem::accessor::kernel::<u32>(LVT_TIMER) };
        let initial_count = unsafe { crate::mem::accessor::kernel::<u32>(INITIAL_COUNT) };
        let current_count = unsafe { crate::mem::accessor::kernel::<u32>(CURRENT_COUNT) };
        let divide_config = unsafe { crate::mem::accessor::kernel::<u32>(DIVIDE_CONFIG) };
        let pm = AcpiPm::new(table);

        Self {
            lvt_timer,
            initial_count,
            current_count,
            divide_config,
            pm,
            frequency: None,
        }
    }

    fn init(&mut self) {
        self.get_frequency();
        self.set_modes();
    }

    fn get_frequency(&mut self) {
        const MAX_COUNT: u32 = !0;

        self.divide_config.write(0b1011);
        self.lvt_timer.write(1 << 16 | 32);
        self.initial_count.write(MAX_COUNT);
        self.pm.wait_milliseconds(100);

        self.frequency = Some((MAX_COUNT - self.current_count.read()) * 10);
    }

    fn set_modes(&mut self) {
        let f = self.frequency.expect("Get the frequency first.");
        info!("Frequency: {}", f);
        self.divide_config.write(3);
        self.lvt_timer.write(u32::from(TIMER_VECTOR) | (1 << 17));
        self.initial_count.write(f * 10);
    }
}

struct AcpiPm {
    reader: Reader,
    supported: SupportedBits,
}
impl AcpiPm {
    pub(crate) fn new(table: &AcpiTables<allocator::acpi::Mapper>) -> Self {
        let pm_timer = table.platform_info().unwrap().pm_timer.unwrap();
        let reader = match pm_timer.base.address_space {
            AddressSpace::SystemMemory => Reader::Memory(MemoryReader::new(table)),
            AddressSpace::SystemIo => Reader::Io(IoReader::new(table)),
            _ => unreachable!(),
        };

        Self {
            reader,
            supported: if pm_timer.supports_32bit {
                SupportedBits::Bits32
            } else {
                SupportedBits::Bits24
            },
        }
    }

    pub(crate) fn wait_milliseconds(&mut self, t: u32) {
        const FREQUENCY: u32 = 3_579_545;
        let start = self.reader.read();
        let mut end = start.wrapping_add(FREQUENCY.wrapping_mul(t / 1000));
        if let SupportedBits::Bits24 = self.supported {
            end &= 0x00ff_ffff;
        }

        if end < start {
            while self.reader.read() >= start {}
        }

        while self.reader.read() < end {}
    }
}

enum Reader {
    Io(IoReader),
    Memory(MemoryReader),
}
impl Reader {
    fn read(&mut self) -> u32 {
        match self {
            Reader::Io(i) => i.read(),
            Reader::Memory(m) => m.read(),
        }
    }
}

struct IoReader {
    // Initialization of the APIC timer is done in the kernel privilege. `syscall` must not be
    // called.
    port: PortReadOnly<u32>,
}
impl IoReader {
    fn new(table: &AcpiTables<allocator::acpi::Mapper>) -> Self {
        let b = table.platform_info().unwrap().pm_timer.unwrap().base;
        Self {
            port: PortReadOnly::new(b.address.try_into().unwrap()),
        }
    }

    fn read(&mut self) -> u32 {
        // SAFETY: This operation is safe as the `port` has an I/O address taken from `AcpiTables`.
        unsafe { self.port.read() }
    }
}

struct MemoryReader {
    addr: Single<u32>,
}
impl MemoryReader {
    fn new(table: &AcpiTables<allocator::acpi::Mapper>) -> Self {
        let b = table.platform_info().unwrap().pm_timer.unwrap().base;
        Self {
            // SAFETY: This operation is safe as the address is generated from `AcpiTables`.
            addr: unsafe { crate::mem::accessor::kernel(PhysAddr::new(b.address)) },
        }
    }

    fn read(&mut self) -> u32 {
        self.addr.read()
    }
}

#[derive(Debug)]
enum SupportedBits {
    Bits32,
    Bits24,
}
