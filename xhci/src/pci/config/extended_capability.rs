use crate::pci::config::{RegisterIndex, Registers};
use bit_field::BitField;
use core::convert::{TryFrom, TryInto};

struct Iter {
    registers: Registers,
    index: RegisterIndex,
}
impl Iter {
    fn new(registers: Registers, index: RegisterIndex) -> Self {
        Self { registers, index }
    }
}
impl Iterator for Iter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let raw = self.registers.get(self.index);
        let next = raw.get_bits(8..=15);

        if next == 0 {
            None
        } else {
            self.index =
                RegisterIndex::new(self.index.as_usize() + usize::try_from(next >> 2).unwrap());

            Some(raw.get_bits(0..=7).try_into().unwrap())
        }
    }
}
