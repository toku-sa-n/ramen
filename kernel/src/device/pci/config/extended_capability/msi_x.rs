// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{RegisterIndex, Registers},
    crate::{accessor::slice, device::pci::config::bar},
    bitfield::bitfield,
    core::convert::{From, TryFrom},
    os_units::{Bytes, Size},
    x86_64::PhysAddr,
};

#[derive(Debug)]
pub struct CapabilitySpec<'a> {
    registers: &'a Registers,
    base: RegisterIndex,
}

impl<'a> CapabilitySpec<'a> {
    pub fn new(registers: &'a Registers, base: RegisterIndex) -> Self {
        Self { registers, base }
    }

    pub fn bir(&self) -> bar::Index {
        bar::Index::from(Bir::new(self.registers, self.base))
    }

    pub fn table(&self, base_address: PhysAddr) -> slice::Accessor<Element> {
        slice::Accessor::new(
            base_address,
            self.table_offset(),
            usize::from(self.num_of_table_elements()),
        )
    }

    pub fn enable_interrupt(&self) {
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
    pub struct Element(u128);

    pub u32, from into MessageAddress, message_address,set_message_address: 31, 0;
    pub u32, from into MessageData, message_data, set_message_data: 95, 64;
    pub masked, set_mask: 96;
}

bitfield! {
    pub struct MessageAddress(u32);

    pub redirection_hint, set_redirection_hint: 3;
    pub u8, destination_id, set_destination_id: 19, 12;
}

impl From<MessageAddress> for u32 {
    fn from(address: MessageAddress) -> Self {
        address.0
    }
}

impl From<u32> for MessageAddress {
    fn from(val: u32) -> Self {
        Self(val)
    }
}

bitfield! {
    pub struct MessageData(u32);

    pub vector, set_vector: 7, 0;
    pub delivery_mode, set_delivery_mode: 10, 8;
    level, set_level: 14;
    trigger_mode, set_trigger_mode: 15;
}

impl MessageData {
    pub fn set_level_trigger(&mut self) {
        self.set_trigger_mode(true);
        self.set_level(true);
    }
}

impl From<MessageData> for u32 {
    fn from(address: MessageData) -> Self {
        address.0
    }
}

impl From<u32> for MessageData {
    fn from(val: u32) -> Self {
        Self(val)
    }
}
