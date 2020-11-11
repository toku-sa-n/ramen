// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::register::{hc_operational::PortRegisters, Registers},
    alloc::{rc::Rc, vec::Vec},
    core::{cell::RefCell, slice},
};

pub struct Collection {
    collection: Vec<Port>,
}
impl<'a> Collection {
    pub fn new(registers: &Rc<RefCell<Registers>>) -> Self {
        let mut collection = Vec::new();
        for i in 0..Self::num_of_ports(&registers) {
            collection.push(Port::new(registers.clone(), i));
        }

        Self { collection }
    }

    pub fn enable_all_connected_ports(&'a mut self) {
        for port in self {
            port.reset_if_connected();
        }
    }

    fn num_of_ports(registers: &Rc<RefCell<Registers>>) -> usize {
        let params1 = &registers.borrow().hc_capability.hcs_params_1;
        params1.read().max_ports().into()
    }
}
impl<'a> IntoIterator for &'a mut Collection {
    type Item = &'a mut Port;
    type IntoIter = slice::IterMut<'a, Port>;

    fn into_iter(self) -> Self::IntoIter {
        self.collection.iter_mut()
    }
}

pub struct Port {
    registers: Rc<RefCell<Registers>>,
    index: usize,
}
impl<'a> Port {
    pub fn reset_if_connected(&mut self) {
        if self.connected() {
            self.reset();
        }
    }

    fn new(registers: Rc<RefCell<Registers>>, index: usize) -> Self {
        Self { registers, index }
    }

    fn connected(&self) -> bool {
        self.read_port_rg().port_sc.current_connect_status()
    }

    fn reset(&mut self) {
        self.start_resetting();
        self.wait_until_reset_completed();
    }

    fn start_resetting(&mut self) {
        let port_rg = &mut self.registers.borrow_mut().hc_operational.port_registers;
        port_rg.update(self.index, |rg| rg.port_sc.set_port_reset(true))
    }

    fn wait_until_reset_completed(&self) {
        while {
            let port_rg = self.read_port_rg();
            !port_rg.port_sc.port_reset_changed()
        } {}
    }

    fn read_port_rg(&self) -> PortRegisters {
        let port_rg = &self.registers.borrow().hc_operational.port_registers;
        port_rg.read(self.index)
    }
}
