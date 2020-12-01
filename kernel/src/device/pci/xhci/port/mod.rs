// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
    exchanger::{command, receiver::Receiver},
    structures::{
        context::Context,
        dcbaa::DeviceContextBaseAddressArray,
        registers::{operational::PortRegisters, Registers},
    },
};
use crate::{
    multitask::task::{self, Task},
    Futurelock,
};
use alloc::rc::Rc;
use core::cell::RefCell;
use endpoint::class_driver;
use resetter::Resetter;
use slot::{endpoint, Slot};

mod context;
mod resetter;
mod slot;

// FIXME
#[allow(clippy::too_many_arguments)]
async fn task(
    mut port: Port,
    runner: Rc<Futurelock<command::Sender>>,
    receiver: Rc<RefCell<Receiver>>,
    task_collection: Rc<RefCell<task::Collection>>,
) {
    port.reset();
    port.init_context();

    let slot_id = runner.lock().await.enable_device_slot().await;

    let mut slot = Slot::new(port, slot_id, receiver);
    slot.init(runner.clone()).await;
    info!("Slot initialized");
    let mut eps = endpoint::Collection::new(slot, runner).await;
    eps.init().await;

    let kbd = class_driver::keyboard::Keyboard::new(eps);
    task_collection
        .borrow_mut()
        .add_task_as_woken(Task::new_poll(class_driver::keyboard::task(kbd)));
}

// FIXME: Resolve this.
#[allow(clippy::too_many_arguments)]
pub fn spawn_tasks(
    command_runner: &Rc<Futurelock<command::Sender>>,
    dcbaa: &Rc<RefCell<DeviceContextBaseAddressArray>>,
    registers: &Rc<RefCell<Registers>>,
    receiver: &Rc<RefCell<Receiver>>,
    task_collection: &Rc<RefCell<task::Collection>>,
) {
    for i in 0..num_of_ports(&registers.borrow()) {
        let port = Port::new(&registers, dcbaa.clone(), i + 1);
        if port.connected() {
            task_collection
                .borrow_mut()
                .add_task_as_woken(Task::new(task(
                    port,
                    command_runner.clone(),
                    receiver.clone(),
                    task_collection.clone(),
                )));
        }
    }
}

fn num_of_ports(registers: &Registers) -> u8 {
    let params1 = registers.capability.hcs_params_1.read();
    params1.max_ports()
}

pub struct Port {
    registers: Rc<RefCell<Registers>>,
    index: u8,
    context: Context,
    dcbaa: Rc<RefCell<DeviceContextBaseAddressArray>>,
}
impl Port {
    fn new(
        registers: &Rc<RefCell<Registers>>,
        dcbaa: Rc<RefCell<DeviceContextBaseAddressArray>>,
        index: u8,
    ) -> Self {
        Self {
            registers: registers.clone(),
            index,
            context: Context::new(&registers.borrow()),
            dcbaa,
        }
    }

    fn connected(&self) -> bool {
        self.read_port_rg().port_sc.current_connect_status()
    }

    fn reset(&mut self) {
        Resetter::new(&mut self.registers.borrow_mut(), self.index).reset();
    }

    fn init_context(&mut self) {
        context::Initializer::new(&mut self.context, self.index).init();
    }

    fn read_port_rg(&self) -> PortRegisters {
        let port_rg = &self.registers.borrow().operational.port_registers;
        port_rg.read((self.index - 1).into())
    }
}
