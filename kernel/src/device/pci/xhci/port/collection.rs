// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{super::register::Registers, Port},
    alloc::vec::Vec,
    core::slice,
    spinning_top::Spinlock,
};

pub struct PortCollection<'a> {
    collection: Vec<Port<'a>>,
}
impl<'a> PortCollection<'a> {
    pub fn new(registers: &'a Spinlock<Registers>) -> Self {
        let mut collection = Vec::new();
        for i in 0..Self::num_of_ports(registers) {
            collection.push(Port::new(registers, i));
        }

        Self { collection }
    }

    fn num_of_ports(registers: &Spinlock<Registers>) -> usize {
        let params1 = &registers.lock().hc_capability.hcs_params_1;
        params1.read().max_ports().into()
    }
}
impl<'a> IntoIterator for &'a mut PortCollection<'a> {
    type Item = &'a mut Port<'a>;
    type IntoIter = slice::IterMut<'a, Port<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.collection.iter_mut()
    }
}
