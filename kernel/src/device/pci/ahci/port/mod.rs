// SPDX-License-Identifier: GPL-3.0-or-later

mod command_list;
mod received_fis;

use super::registers::{port, Registers};
use crate::multitask::task::{self, Task};
use alloc::sync::Arc;
use command_list::CommandList;
use core::convert::TryInto;
use received_fis::ReceivedFis;
use spinning_top::Spinlock;

const MAX_PORTS: usize = 32;

pub fn spawn_tasks(registers: &Arc<Spinlock<Registers>>) {
    (0..MAX_PORTS)
        .filter_map(|i| Port::new(registers.clone(), i))
        .for_each(|port| {
            task::COLLECTION
                .lock()
                .add_task_as_woken(Task::new(task(port)))
        })
}

async fn task(mut port: Port) {
    info!("This is a task of port {}", port.index);
    port.init();
    port.start();

    info!(
        "Available command slot: {:?}",
        port.get_available_command_slot()
    );
}

pub struct Port {
    registers: Arc<Spinlock<Registers>>,
    command_list: CommandList,
    received_fis: ReceivedFis,
    index: usize,
}
impl Port {
    fn idle(&mut self) {
        self.edit_port_rg(|rg| {
            rg.cmd.update(|cmd| {
                cmd.set_start_bit(false);
                cmd.set_fis_receive_enable(false)
            })
        });

        while {
            self.parse_port_rg(|reg| {
                let cmd = reg.cmd.read();
                cmd.command_list_running() || cmd.fis_receive_running()
            })
        } {}
    }

    fn new(registers: Arc<Spinlock<Registers>>, index: usize) -> Option<Self> {
        if Self::exists(&registers, index) {
            Some(Self::generate(registers, index))
        } else {
            None
        }
    }

    fn exists(registers: &Arc<Spinlock<Registers>>, index: usize) -> bool {
        let registers = &registers.lock();
        let pi: usize = registers.generic.pi.read().0.try_into().unwrap();
        pi & (1 << index) != 0
    }

    fn generate(registers: Arc<Spinlock<Registers>>, index: usize) -> Self {
        let command_list = CommandList::new(&*registers.lock());
        let received_fis = ReceivedFis::new();
        Self {
            registers,
            received_fis,
            command_list,
            index,
        }
    }

    fn init(&mut self) {
        self.idle();
        self.register_command_list_and_received_fis();
        self.clear_error_bits();
    }

    fn register_command_list_and_received_fis(&mut self) {
        self.assert_64bit_accessing_is_supported();
        self.register_command_list();
        self.register_received_fis();
    }

    fn assert_64bit_accessing_is_supported(&self) {
        let registers = &self.registers.lock();
        assert!(registers.generic.cap.read().supports_64bit_addressing());
    }

    fn register_command_list(&mut self) {
        let addr = self.command_list.phys_addr_to_headers();
        self.edit_port_rg(|rg| rg.clb.update(|b| b.set(addr)));
    }

    fn register_received_fis(&mut self) {
        self.register_fis_addr();
        self.enable_receiving_fis();
    }

    fn register_fis_addr(&mut self) {
        let addr = self.received_fis.phys_addr();
        self.edit_port_rg(|rg| rg.fb.update(|b| b.set(addr)));
    }

    fn enable_receiving_fis(&mut self) {
        self.edit_port_rg(|r| r.cmd.update(|r| r.set_fis_receive_enable(true)));
    }

    fn clear_error_bits(&mut self) {
        // Refer to P.31 and P.104 of Serial ATA AHCI 1.3.1 Specification
        const BIT_MASK: u32 = 0x07ff_0f03;
        self.edit_port_rg(|rg| rg.serr.update(|serr| serr.0 = BIT_MASK));
    }

    fn start(&mut self) {
        if self.ready_to_start() {
            self.start_processing();
            info!(
                "Port {} signature: {:X}.",
                self.index,
                self.parse_port_rg(|r| r.sig.read().get())
            );
        }
    }

    fn ready_to_start(&self) -> bool {
        !self.command_list_is_running() && self.fis_receive_enabled() && self.device_is_present()
    }

    fn command_list_is_running(&self) -> bool {
        self.parse_port_rg(|r| r.cmd.read().command_list_running())
    }

    fn fis_receive_enabled(&self) -> bool {
        self.parse_port_rg(|r| r.cmd.read().fis_receive_enable())
    }

    fn device_is_present(&self) -> bool {
        self.parse_port_rg(|r| {
            r.ssts.read().device_detection() == 3
                || [2, 6, 8].contains(&r.ssts.read().interface_power_management())
        })
    }

    fn start_processing(&mut self) {
        self.edit_port_rg(|r| r.cmd.update(|r| r.set_start_bit(true)))
    }

    fn get_available_command_slot(&self) -> Option<usize> {
        for i in 0..self.num_of_command_slots() {
            if self.slot_available(i) {
                return Some(i);
            }
        }

        None
    }

    fn num_of_command_slots(&self) -> usize {
        let cap = &self.registers.lock().generic.cap.read();
        cap.num_of_command_slots().try_into().unwrap()
    }

    fn slot_available(&self, index: usize) -> bool {
        let bitflag_used_slots = self.parse_port_rg(|r| r.sact.read().get() | r.ci.read().get());
        bitflag_used_slots & (1 << index) == 0
    }

    fn parse_port_rg<T, U>(&self, f: T) -> U
    where
        T: Fn(&port::Registers) -> U,
    {
        let registers = &self.registers.lock();
        let port_rg = registers.port_regs[self.index].as_ref().unwrap();
        f(port_rg)
    }

    fn edit_port_rg<T>(&mut self, f: T)
    where
        T: Fn(&mut port::Registers),
    {
        let registers = &mut self.registers.lock();
        let port_rg = registers.port_regs[self.index].as_mut().unwrap();
        f(port_rg);
    }
}
