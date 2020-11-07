// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::register::{hc_operational::PortRegisters, Registers},
    alloc::vec::Vec,
    spinning_top::Spinlock,
};

pub struct Port<'a> {
    registers: &'a Spinlock<Registers>,
    index: usize,
}
impl<'a> Port<'a> {
    pub fn new_all_ports(registers: &'a Spinlock<Registers>) -> Vec<Self> {
        let mut ports = Vec::new();
        for i in 0..Self::num_of_ports(registers) {
            ports.push(Self::new(registers, i));
        }

        ports
    }

    pub fn reset_if_connected(&mut self) {
        if self.connected() {
            self.reset();
        }
    }

    fn num_of_ports(registers: &Spinlock<Registers>) -> usize {
        let params1 = &registers.lock().hc_capability.hcs_params_1;
        params1.read().max_ports().into()
    }

    fn new(registers: &'a Spinlock<Registers>, index: usize) -> Self {
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
        let port_rg = &mut self.registers.lock().hc_operational.port_registers;
        port_rg.update(self.index, |rg| rg.port_sc.set_port_reset(true))
    }

    fn wait_until_reset_completed(&self) {
        while {
            let port_rg = self.read_port_rg();
            !port_rg.port_sc.port_reset_changed()
        } {}
    }

    fn read_port_rg(&self) -> PortRegisters {
        let port_rg = &self.registers.lock().hc_operational.port_registers;
        port_rg.read(self.index)
    }
}
