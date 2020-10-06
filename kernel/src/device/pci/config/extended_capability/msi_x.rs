// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{MessageAddress, MessageData, RegisterIndex, Registers},
    crate::{
        accessor::slice,
        device::pci::config::{bar, type_spec::TypeSpec},
    },
    bitfield::bitfield,
    common::constant::LOCAL_APIC_ID_REGISTER_ADDR,
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

    fn init_for_xhci(&self, config_type_spec: &TypeSpec) {
        let base_address = config_type_spec.base_address(self.bir().into());
        let mut table = self.table(base_address);

        table[0]
            .message_address()
            .set_destination_id(Self::get_local_apic_id());
        table[0].message_address().set_redirection_hint(true);
        table[0].message_data().set_level_trigger();
        table[0].message_data().set_vector(0x40);
        table[0].set_mask(false);

        self.enable_interrupt();
    }

    fn get_local_apic_id() -> u8 {
        u8::try_from(unsafe { *(LOCAL_APIC_ID_REGISTER_ADDR.as_ptr() as *const u32) } >> 24)
            .unwrap()
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
