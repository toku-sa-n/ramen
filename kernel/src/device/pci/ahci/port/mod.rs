// SPDX-License-Identifier: GPL-3.0-or-later

mod command_list;
mod received_fis;

use {
    super::registers::{port, Registers},
    alloc::{rc::Rc, vec::Vec},
    command_list::CommandList,
    core::{cell::RefCell, convert::TryInto},
    received_fis::ReceivedFis,
};

pub struct Collection(Vec<Port>);
impl Collection {
    const MAX_PORTS: usize = 32;

    pub fn new(registers: &Rc<RefCell<Registers>>) -> Self {
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

    pub fn clear_error_bits(&mut self) {
        for port in self.iter_mut() {
            port.clear_error_bits();
        }
    }

    pub fn register_command_lists_and_fis(&mut self) {
        for port in self.iter_mut() {
            port.register_command_list_and_received_fis();
        }
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Port> {
        self.0.iter_mut()
    }
}

pub struct Port {
    registers: Rc<RefCell<Registers>>,
    command_list: CommandList,
    received_fis: ReceivedFis,
    index: usize,
}
impl Port {
    pub fn idle(&mut self) {
        self.edit_port_rg(|rg| {
            rg.px_cmd.update(|cmd| {
                cmd.set_start_bit(false);
                cmd.set_fis_receive_enable(false)
            })
        });

        while {
            self.parse_port_rg(|reg| {
                let cmd = reg.px_cmd.read();
                cmd.command_list_running() || cmd.fis_receive_running()
            })
        } {}
    }

    fn new(registers: Rc<RefCell<Registers>>, index: usize) -> Option<Self> {
        if Self::exists(&registers, index) {
            Some(Self::generate(registers, index))
        } else {
            None
        }
    }

    fn exists(registers: &Rc<RefCell<Registers>>, index: usize) -> bool {
        let registers = &registers.borrow();
        let pi: usize = registers.generic.pi.read().0.try_into().unwrap();
        pi & (1 << index) != 0
    }

    fn generate(registers: Rc<RefCell<Registers>>, index: usize) -> Self {
        let command_list = CommandList::new(&*registers.borrow());
        let received_fis = ReceivedFis::new();
        Self {
            registers,
            received_fis,
            command_list,
            index,
        }
    }

    fn register_command_list_and_received_fis(&mut self) {
        self.assert_64bit_accessing_is_supported();
        self.register_command_list();
        self.register_received_fis();
    }

    fn assert_64bit_accessing_is_supported(&self) {
        let registers = &self.registers.borrow();
        assert!(registers.generic.cap.read().supports_64bit_addressing());
    }

    fn register_command_list(&mut self) {
        let addr = self.command_list.phys_addr();
        self.edit_port_rg(|rg| rg.px_clb.update(|b| b.set(addr)));
    }

    fn register_received_fis(&mut self) {
        let addr = self.received_fis.phys_addr();
        self.edit_port_rg(|rg| rg.px_fb.update(|b| b.set(addr)));
    }

    fn clear_error_bits(&mut self) {
        // Refer to P.31 and P.104 of Serial ATA AHCI 1.3.1 Specification
        const BIT_MASK: u32 = 0x07ff_0f03;
        self.edit_port_rg(|rg| rg.px_serr.update(|serr| serr.0 = BIT_MASK));
    }

    fn read_port_rg(&self) {
        let registers = &self.registers.borrow();
        let port_rg = registers.port_regs[self.index].as_ref().unwrap();
    }

    fn parse_port_rg<T, U>(&self, f: T) -> U
    where
        T: Fn(&port::Registers) -> U,
    {
        let registers = &self.registers.borrow_mut();
        let port_rg = registers.port_regs[self.index].as_ref().unwrap();
        f(port_rg)
    }

    fn edit_port_rg<T>(&mut self, f: T)
    where
        T: Fn(&mut port::Registers),
    {
        let registers = &mut self.registers.borrow_mut();
        let port_rg = registers.port_regs[self.index].as_mut().unwrap();
        f(port_rg);
    }
}
