// SPDX-License-Identifier: GPL-3.0-or-later

use crate::mem::{accessor::Accessor, allocator};
use acpi::{
    platform::{Apic, IoApic},
    AcpiTables, InterruptModel,
};
use alloc::vec::Vec;
use bit_field::BitField;
use core::convert::TryInto;
use os_units::Bytes;
use x86_64::{instructions::port::Port, PhysAddr};

/// Currently this OS does not support multiple I/O APIC.

struct Registers {
    addr: Accessor<u32>,
    data: Accessor<u32>,
}
impl Registers {
    const DEST_BASE: u8 = 0x10;

    /// Safety: This operation is unsafe because the caller must ensure that `IoApic::address` must
    /// be a valid address to I/O APIC registers.
    ///
    /// There is no need to create an instance of `IoApic` manually, but because it is possible as
    /// the all fields of the struct are public, this method is unsafe.
    unsafe fn new(io_apics: Vec<IoApic>) -> Self {
        let io_apic_base = PhysAddr::new(io_apics[0].address.into());

        Self {
            addr: Accessor::new(io_apic_base, Bytes::new(0)),
            data: Accessor::new(io_apic_base, Bytes::new(0x10)),
        }
    }

    fn set_redirection(&mut self, irq: u8, redirection: Redirection) {
        let val = redirection.as_u64();
        let l = val & 0xffff_ffff;
        let u = val >> 32;

        self.write(Self::DEST_BASE + irq * 2, l.try_into().unwrap());
        self.write(Self::DEST_BASE + irq * 2 + 1, u.try_into().unwrap());
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
        self.addr.write(index.into());
        self.data.write(v);
    }
}

#[derive(Builder)]
#[builder(no_std)]
struct Redirection {
    vec: u8,
    delivery: Delivery,
    destination: DestinationMode,
    polarity: Polarity,
    trigger: TriggerMode,
    mask: bool,
}
impl Redirection {
    fn as_u64(&self) -> u64 {
        let mut v = 0_u64;
        v.set_bits(0..=7, self.vec.into());
        v.set_bits(8..=10, self.delivery as u64);
        v.set_bit(11, self.destination.as_bool());
        v.set_bit(13, self.polarity.as_bool());
        v.set_bit(15, self.trigger.as_bool());
        v.set_bit(16, self.mask);

        match self.destination {
            DestinationMode::Physical(p) => v.set_bits(56..=59, p.into()),
            DestinationMode::Logical(l) => v.set_bits(56..=63, l.into()),
        };

        v
    }
}

#[derive(Copy, Clone)]
enum Delivery {
    Normal = 0,
    LowPriority = 1,
    SystemManagementInterrupt = 2,
    NonMaskableInterrupt = 4,
    Init = 5,
    External = 7,
}

#[derive(Clone)]
enum DestinationMode {
    Physical(u8),
    Logical(u8),
}
impl DestinationMode {
    fn as_bool(&self) -> bool {
        match self {
            Self::Physical(_) => false,
            Self::Logical(_) => true,
        }
    }
}

#[derive(Clone)]
enum Polarity {
    HighIsActive = 0,
    LowIsActive = 1,
}
impl Polarity {
    fn as_bool(&self) -> bool {
        match self {
            Self::HighIsActive => false,
            Self::LowIsActive => true,
        }
    }
}

#[derive(Clone)]
enum TriggerMode {
    Edge = 0,
    Level = 1,
}
impl TriggerMode {
    fn as_bool(&self) -> bool {
        match self {
            Self::Edge => false,
            Self::Level => true,
        }
    }
}

/// Safety: This method is unsafe because the caller must ensure that `rsdb` is a valid RSDB.
/// Otherwise this function will break memory safety by dereferencing to an invalid address.
pub unsafe fn init(rsdb: PhysAddr) {
    disable_pic();
    let table = fetch_apic(rsdb);
    let platform_info = table.platform_info().unwrap();
    let interrupt = platform_info.interrupt_model;
    if let InterruptModel::Apic(apic) = interrupt {
        let id = apic.io_apics[0].id;
        info!("{:?}", apic);
        let mut registers = Registers::new(apic.io_apics);
        registers.mask_all();
        init_ps2_keyboard(&mut registers, id);
        init_ps2_mouse(&mut registers, id);
    }
    x86_64::instructions::interrupts::enable();
}

fn disable_pic() {
    pic_init_mode();
    remap_pic();
    set_slave_offset();
    nonbuffer_mode();
    mask_pic();
}

fn pic_init_mode() {
    const PIC0_ICW1: u16 = 0x0020;
    const PIC1_ICW1: u16 = 0x00A0;

    unsafe {
        Port::new(PIC0_ICW1).write(0x11_u8);
        Port::new(PIC1_ICW1).write(0x11_u8);
    }
}

fn remap_pic() {
    const PIC0_ICW2: u16 = 0x0021;
    const PIC1_ICW2: u16 = 0x00A1;

    unsafe {
        Port::new(PIC0_ICW2).write(0x20_u8);
        Port::new(PIC1_ICW2).write(0x28_u8);
    }
}

fn set_slave_offset() {
    const PIC0_ICW3: u16 = 0x0021;
    const PIC1_ICW3: u16 = 0x00A1;

    unsafe {
        Port::new(PIC0_ICW3).write(4_u8);
        Port::new(PIC1_ICW3).write(2_u8);
    }
}

fn nonbuffer_mode() {
    const PIC0_ICW4: u16 = 0x0021;
    const PIC1_ICW4: u16 = 0x00A1;

    unsafe {
        Port::new(PIC0_ICW4).write(1_u8);
        Port::new(PIC1_ICW4).write(1_u8);
    }
}

fn mask_pic() {
    const PIC0_IMR: u16 = 0x0021;
    const PIC1_IMR: u16 = 0x00A1;

    // Safety: These operations are safe because `PIC0_IMR` and `PIC1_IMR` are the valid port numbers.
    unsafe {
        Port::new(PIC0_IMR).write(0xFF_u8);
        Port::new(PIC1_IMR).write(0xFF_u8);
    }
}

/// Safety: This method is unsafe because the caller must ensure that `rsdb` is a valid RSDB.
/// Otherwise this function will break memory safety by dereferencing to an invalid address.
unsafe fn fetch_apic(rsdb: PhysAddr) -> AcpiTables<allocator::acpi::Mapper> {
    let mapper = allocator::acpi::Mapper;
    AcpiTables::from_rsdp(mapper, rsdb.as_u64().try_into().unwrap()).unwrap()
}

fn init_ps2_keyboard(r: &mut Registers, apic_id: u8) {
    let key = RedirectionBuilder::default()
        .vec(0x21)
        .delivery(Delivery::Normal)
        .destination(DestinationMode::Physical(apic_id))
        .polarity(Polarity::HighIsActive)
        .trigger(TriggerMode::Edge)
        .mask(false)
        .build()
        .unwrap();

    r.set_redirection(1, key);
}

fn init_ps2_mouse(r: &mut Registers, apic_id: u8) {
    let mouse = RedirectionBuilder::default()
        .vec(0x2c)
        .delivery(Delivery::Normal)
        .destination(DestinationMode::Physical(apic_id))
        .polarity(Polarity::HighIsActive)
        .trigger(TriggerMode::Edge)
        .mask(false)
        .build()
        .unwrap();

    r.set_redirection(12, mouse);
}
