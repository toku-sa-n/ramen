// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::pic,
    crate::mem::{accessor::Single, allocator},
    acpi::{platform::interrupt::IoApic, AcpiTables, InterruptModel},
    x86_64::PhysAddr,
};

/// Currently this OS does not support multiple I/O APIC.

struct Registers {
    addr: Single<u32>,
    data: Single<u32>,
}
impl Registers {
    const DEST_BASE: u8 = 0x10;

    /// SAFETY: This operation is unsafe because the caller must ensure that `IoApic::address` must
    /// be a valid address to I/O APIC registers.
    ///
    /// There is no need to create an instance of `IoApic` manually, but because it is possible as
    /// the all fields of the struct are public, this method is unsafe.
    ///
    /// This method must be called in the kernel privilege.
    unsafe fn new(io_apics: &[IoApic]) -> Self {
        let io_apic_base = PhysAddr::new(io_apics[0].address.into());

        // SAFETY: The caller must ensure that `io_apics[0]`.address` is the correct address.
        unsafe {
            Self {
                addr: crate::mem::accessor::new(io_apic_base),
                data: crate::mem::accessor::new(io_apic_base + 0x10_usize),
            }
        }
    }

    fn mask_all(&mut self) {
        const MAX_IRQ: u8 = 24;

        for i in 0..MAX_IRQ {
            self.mask(i);
        }
    }

    fn mask(&mut self, irq: u8) {
        const MASK_INTERRUPT: u32 = 0x100;

        self.write(Self::DEST_BASE + irq * 2, MASK_INTERRUPT);
    }

    fn write(&mut self, index: u8, v: u32) {
        self.addr.write_volatile(index.into());
        self.data.write_volatile(v);
    }
}

pub(crate) fn init(table: &AcpiTables<allocator::acpi::Mapper>) {
    pic::disable();
    let platform_info = table.platform_info().unwrap();
    let interrupt = platform_info.interrupt_model;
    if let InterruptModel::Apic(apic) = interrupt {
        // SAFETY: This operation is safe because `table` contains valid information.
        let mut registers = unsafe { Registers::new(&apic.io_apics) };
        registers.mask_all();
    }
}
