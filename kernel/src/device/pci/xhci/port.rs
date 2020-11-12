// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{
        command_runner::Runner,
        register::{hc_operational::PortRegisters, Registers},
    },
    crate::multitask::task::{self, Task},
    alloc::rc::Rc,
    core::cell::RefCell,
    futures_intrusive::sync::LocalMutex,
};

async fn task(mut port: Port, command_runner: Rc<LocalMutex<Runner>>) {
    info!("This is a task of port {}", port.index);
    port.reset_if_connected();
    command_runner.lock().await.noop().await.unwrap();
    info!("NOOP");
}

pub struct TaskSpawner {
    command_runner: Rc<LocalMutex<Runner>>,
    registers: Rc<RefCell<Registers>>,
    task_collection: Rc<RefCell<task::Collection>>,
}
impl<'a> TaskSpawner {
    pub fn new(
        command_runner: Rc<LocalMutex<Runner>>,
        registers: Rc<RefCell<Registers>>,
        task_collection: Rc<RefCell<task::Collection>>,
    ) -> Self {
        Self {
            command_runner,
            registers,
            task_collection,
        }
    }

    pub fn spawn_tasks(&self) {
        for i in 0..self.num_of_ports() {
            let port = Port::new(self.registers.clone(), i);
            if port.connected() {
                self.task_collection
                    .borrow_mut()
                    .add_task_as_woken(Task::new(task(port, self.command_runner.clone())));
            }
        }
    }

    fn num_of_ports(&self) -> usize {
        let params1 = &self.registers.borrow().hc_capability.hcs_params_1;
        params1.read().max_ports().into()
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
