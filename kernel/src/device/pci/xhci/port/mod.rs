// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
    exchanger::command::Sender,
    structures::{
        context::Context,
        dcbaa::DeviceContextBaseAddressArray,
        registers::{operational::PortRegisters, Registers},
        ring::transfer,
    },
};
use crate::multitask::task::{self, Task};
use alloc::rc::Rc;
use core::{cell::RefCell, convert::TryInto};
use futures_intrusive::sync::LocalMutex;
use resetter::Resetter;
use x86_64::PhysAddr;

mod context;
mod resetter;

async fn task(mut port: Port, runner: Rc<LocalMutex<Sender>>) {
    port.reset();

    let slot_id = runner.lock().await.enable_device_slot().await;

    port.init_device_slot(slot_id, runner).await;
}

// FIXME: Resolve this.
#[allow(clippy::too_many_arguments)]
pub fn spawn_tasks(
    command_runner: &Rc<LocalMutex<Sender>>,
    dcbaa: &Rc<RefCell<DeviceContextBaseAddressArray>>,
    registers: &Rc<RefCell<Registers>>,
    task_collection: &Rc<RefCell<task::Collection>>,
) {
    for i in 0..num_of_ports(&registers) {
        let port = Port::new(&registers, dcbaa.clone(), i + 1);
        if port.connected() {
            task_collection
                .borrow_mut()
                .add_task_as_woken(Task::new(task(port, command_runner.clone())));
        }
    }
}

fn num_of_ports(registers: &Rc<RefCell<Registers>>) -> usize {
    let params1 = registers.borrow().capability.hcs_params_1.read();
    params1.max_ports().into()
}

pub struct Port {
    registers: Rc<RefCell<Registers>>,
    index: usize,
    context: Context,
    transfer_ring: transfer::Ring,
    dcbaa: Rc<RefCell<DeviceContextBaseAddressArray>>,
}
impl Port {
    fn new(
        registers: &Rc<RefCell<Registers>>,
        dcbaa: Rc<RefCell<DeviceContextBaseAddressArray>>,
        index: usize,
    ) -> Self {
        Self {
            registers: registers.clone(),
            index,
            context: Context::new(&registers.borrow()),
            dcbaa,
            transfer_ring: transfer::Ring::new(),
        }
    }

    fn connected(&self) -> bool {
        self.read_port_rg().port_sc.current_connect_status()
    }

    fn reset(&mut self) {
        Resetter::new(self.registers.clone(), self.index).reset();
    }

    async fn init_device_slot(&mut self, slot_id: u8, runner: Rc<LocalMutex<Sender>>) {
        context::Initializer::new(
            &mut self.context,
            &self.transfer_ring,
            self.index.try_into().unwrap(),
        )
        .init();
        self.register_to_dcbaa(slot_id.into());

        runner
            .lock()
            .await
            .address_device(self.addr_to_input_context(), slot_id)
            .await;
    }

    fn addr_to_input_context(&self) -> PhysAddr {
        self.context.input.phys_addr()
    }

    fn register_to_dcbaa(&mut self, slot_id: usize) {
        self.dcbaa.borrow_mut()[slot_id] = self.context.output_device.phys_addr();
    }

    fn read_port_rg(&self) -> PortRegisters {
        let port_rg = &self.registers.borrow().operational.port_registers;
        port_rg.read(self.index - 1)
    }
}
