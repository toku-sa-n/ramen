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
    multitask::{self, task::Task},
    Futurelock,
};
use alloc::sync::Arc;
use endpoint::class_driver;
use resetter::Resetter;
use slot::{endpoint, Slot};
use spinning_top::Spinlock;

mod context;
mod resetter;
mod slot;

async fn task(
    mut port: Port,
    runner: Arc<Futurelock<command::Sender>>,
    receiver: Arc<Spinlock<Receiver>>,
) {
    port.reset();
    port.init_context();

    let slot_id = runner.lock().await.enable_device_slot().await;

    let mut slot = Slot::new(port, slot_id, receiver);
    slot.init(runner.clone()).await;
    debug!("Slot initialized");
    let mut eps = endpoint::Collection::new(slot, runner).await;
    eps.init().await;

    let kbd = class_driver::keyboard::Keyboard::new(eps);
    multitask::add(Task::new_poll(class_driver::keyboard::task(kbd)));
}

// FIXME: Resolve this.
#[allow(clippy::too_many_arguments)]
pub fn spawn_tasks(
    command_runner: &Arc<Futurelock<command::Sender>>,
    dcbaa: &Arc<Spinlock<DeviceContextBaseAddressArray>>,
    registers: &Arc<Spinlock<Registers>>,
    receiver: &Arc<Spinlock<Receiver>>,
) {
    let ports_num = num_of_ports(&registers.lock());
    for i in 0..ports_num {
        let port = Port::new(&registers, dcbaa.clone(), i + 1);
        if port.connected() {
            multitask::add(Task::new(task(
                port,
                command_runner.clone(),
                receiver.clone(),
            )));
        }
    }
}

fn num_of_ports(registers: &Registers) -> u8 {
    let params1 = registers.capability.hcs_params_1.read();
    params1.max_ports()
}

pub struct Port {
    registers: Arc<Spinlock<Registers>>,
    index: u8,
    context: Context,
    dcbaa: Arc<Spinlock<DeviceContextBaseAddressArray>>,
}
impl Port {
    fn new(
        registers: &Arc<Spinlock<Registers>>,
        dcbaa: Arc<Spinlock<DeviceContextBaseAddressArray>>,
        index: u8,
    ) -> Self {
        Self {
            registers: registers.clone(),
            index,
            context: Context::new(&registers.lock()),
            dcbaa,
        }
    }

    fn connected(&self) -> bool {
        self.read_port_rg().port_sc.current_connect_status()
    }

    fn reset(&mut self) {
        Resetter::new(&mut self.registers.lock(), self.index).reset();
    }

    fn init_context(&mut self) {
        context::Initializer::new(&mut self.context, self.index).init();
    }

    fn read_port_rg(&self) -> PortRegisters {
        let port_rg = &self.registers.lock().operational.port_registers;
        port_rg.read((self.index - 1).into())
    }
}
