// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::registers::Registers,
    alloc::{rc::Rc, vec::Vec},
    core::cell::RefCell,
    core::convert::TryInto,
};

pub struct PortCollection(Vec<Port>);
impl PortCollection {
    const MAX_PORTS: usize = 32;

    pub fn new(registers: Rc<RefCell<Registers>>) -> Self {
        Self(
            (0..Self::MAX_PORTS)
                .filter_map(|i| Port::new(registers.clone(), i))
                .collect(),
        )
    }

    pub fn idle_all_ports(&mut self) {
        for port in self.iter_mut() {
            port.idle();
        }
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Port> {
        self.0.iter_mut()
    }
}

pub struct Port {
    registers: Rc<RefCell<Registers>>,
    index: usize,
}
impl Port {
    pub fn idle(&mut self) {
        let registers = &mut self.registers.borrow_mut();
        let port_rg = &mut registers.port_regs[self.index].as_mut().unwrap();
        let px_cmd = &mut port_rg.px_cmd;

        px_cmd.update(|cmd| cmd.set_start_bit(false));
        while px_cmd.read().command_list_running() {}

        px_cmd.update(|cmd| cmd.set_fis_receive_enable(false));
        while px_cmd.read().fis_receive_running() {}
    }

    fn new(registers: Rc<RefCell<Registers>>, index: usize) -> Option<Self> {
        if Self::exists(&registers, index) {
            Some(Self { registers, index })
        } else {
            None
        }
    }

    fn exists(registers: &Rc<RefCell<Registers>>, index: usize) -> bool {
        let registers = &registers.borrow();
        let pi: usize = registers.generic.pi.read().0.try_into().unwrap();
        pi & (1 << index) != 0
    }
}
