use crate::{
    msi_x::MsiX,
    pci::config::{RegisterIndex, Registers},
};
use bit_field::BitField;
use core::convert::TryFrom;

pub(super) struct Iter<'a> {
    registers: &'a Registers,
    index: Option<RegisterIndex>,
}
impl<'a> Iter<'a> {
    pub(super) fn new(registers: &'a Registers, index: RegisterIndex) -> Self {
        Self {
            registers,
            index: Some(index),
        }
    }
}
impl<'a> Iterator for Iter<'a> {
    type Item = Option<MsiX<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index?;

        let raw = self.registers.get(index);

        let id = raw.get_bits(0..=7);

        let next = raw.get_bits(8..=15);
        self.index = (next != 0).then(|| index + usize::try_from(next >> 2).unwrap());

        Some(MsiX::new(self.registers, index))
    }
}
