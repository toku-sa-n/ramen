use crate::pci::config::{RegisterIndex, Registers};
use bit_field::BitField;
use core::convert::TryInto;

pub(crate) struct MsiX<'a> {
    registers: &'a Registers,
    base: RegisterIndex,
}
impl<'a> MsiX<'a> {
    pub(crate) fn new(registers: &'a Registers, base: RegisterIndex) -> Option<Self> {
        let ty = registers.get(base).get_bits(0..=7);
        (ty == 0x11).then(|| Self { registers, base })
    }

    fn table_size(&self) -> u16 {
        let v = self.registers.get(self.base);
        v.get_bits(0..=10).try_into().unwrap()
    }

    fn enable_interrupts(&self) {
        self.registers.edit(self.base, |raw| {
            raw.set_bit(31, true);
        });
    }

    fn table_offset(&self) -> u32 {
        let v = self.registers.get(self.base + 1);
        v & !0b111
    }

    fn bir(&self) -> u8 {
        let v = self.registers.get(self.base + 1);
        v.get_bits(0..=2).try_into().unwrap()
    }
}
