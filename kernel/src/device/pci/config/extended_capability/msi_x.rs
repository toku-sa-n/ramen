// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{CapabilitySpec, MessageAddress, MessageData, RegisterIndex, Registers},
    crate::{
        accessor::slice,
        device::pci::config::{bar, type_spec::TypeSpec},
    },
    bitfield::bitfield,
    core::convert::{From, TryFrom},
    os_units::{Bytes, Size},
    x86_64::PhysAddr,
};

#[derive(Debug)]
pub struct MsiX<'a> {
    registers: &'a Registers,
    base: RegisterIndex,
}

impl<'a> CapabilitySpec for MsiX<'a> {
    fn init_for_xhci(&self, config_type_spec: &TypeSpec) {
        let base_address = config_type_spec.base_address(self.bir());
        let mut table = self.table(base_address);

        table[0].init_for_xhci();

        self.enable_interrupt();
    }
}

impl<'a> MsiX<'a> {
    pub fn new(registers: &'a Registers, base: RegisterIndex) -> Self {
        Self { registers, base }
    }

    fn bir(&self) -> bar::Index {
        bar::Index::from(Bir::new(self.registers, self.base))
    }

    fn table(&self, base_address: PhysAddr) -> slice::Accessor<Element> {
        slice::Accessor::new(
            base_address,
            self.table_offset(),
            usize::from(self.num_of_table_elements()),
        )
    }

    fn enable_interrupt(&self) {
        let val = self.registers.get(self.base) | 0x8000_0000;
        self.registers.set(self.base, val);
    }

    fn table_offset(&self) -> Size<Bytes> {
        Size::from(TableOffset::new(self.registers, self.base))
    }

    fn num_of_table_elements(&self) -> TableSize {
        TableSize::new(self.registers, self.base)
    }
}

pub struct Bir(bar::Index);
impl Bir {
    fn new(registers: &Registers, base: RegisterIndex) -> Self {
        Self(bar::Index::new(registers.get(base + 1) & 0b111))
    }
}
impl From<Bir> for bar::Index {
    fn from(bir: Bir) -> Self {
        bir.0
    }
}

struct TableOffset(Size<Bytes>);
impl TableOffset {
    fn new(raw: &Registers, base: RegisterIndex) -> Self {
        Self(Size::new((raw.get(base + 4) & !0xf) as usize))
    }
}
impl From<TableOffset> for Size<Bytes> {
    fn from(offset: TableOffset) -> Self {
        offset.0
    }
}

#[derive(Debug)]
struct TableSize(u32);
impl TableSize {
    fn new(raw: &Registers, base: RegisterIndex) -> Self {
        // Table size is N - 1 encoded.
        // See: https://wiki.osdev.org/PCI#Enabling_MSI-X
        Self(((raw.get(base) >> 16) & 0x7ff) + 1)
    }
}
impl From<TableSize> for usize {
    fn from(size: TableSize) -> Self {
        usize::try_from(size.0).unwrap()
    }
}

bitfield! {
    #[derive(Debug)]
    #[repr(transparent)]
    struct Element(u128);

    u32, from into MessageAddress, message_address,set_message_address: 31, 0;
    u32, from into MessageData, message_data, set_message_data: 95, 64;
    masked, set_mask: 96;
}
impl Element {
    fn init_for_xhci(&mut self) {
        self.message_address().init_for_xhci();
        self.message_data().init_for_xhci();
        self.set_mask(false);
    }
}
