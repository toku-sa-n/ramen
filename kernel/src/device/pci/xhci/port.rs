// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
    exchanger::command::Sender,
    structures::{
        context::{self, EndpointType},
        dcbaa::DeviceContextBaseAddressArray,
        registers::{operational::PortRegisters, Registers},
        ring::transfer,
    },
};
use crate::multitask::task::{self, Task};
use alloc::rc::Rc;
use context::Context;
use core::{cell::RefCell, convert::TryInto};
use futures_intrusive::sync::LocalMutex;
use x86_64::PhysAddr;

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
        self.init_input_context();
        self.init_input_slot_context();
        self.init_input_default_control_endpoint0_context();
        self.register_to_dcbaa(slot_id.into());

        runner
            .lock()
            .await
            .address_device(self.addr_to_input_context(), slot_id)
            .await;
    }

    fn init_input_context(&mut self) {
        let input_control = self.context.input.control_mut();
        input_control.set_aflag(0);
        input_control.set_aflag(1);
    }

    fn init_input_slot_context(&mut self) {
        let slot = &mut self.context.input.device_mut().slot;
        slot.set_context_entries(1);
        slot.set_root_hub_port_number(self.index.try_into().unwrap());
    }

    fn init_input_default_control_endpoint0_context(&mut self) {
        let ep_0 = &mut self.context.input.device_mut().ep_0;
        ep_0.set_endpoint_type(EndpointType::Control);
        // FIXME
        ep_0.set_max_packet_size(64);
        ep_0.set_dequeue_ptr(self.transfer_ring.phys_addr());
        ep_0.set_dequeue_cycle_state(true);
        ep_0.set_error_count(3);
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

struct Resetter {
    registers: Rc<RefCell<Registers>>,
    slot: usize,
}
impl Resetter {
    fn new(registers: Rc<RefCell<Registers>>, slot: usize) -> Self {
        Self { registers, slot }
    }

    fn reset(&mut self) {
        self.start_resetting();
        self.wait_until_reset_is_completed();
    }

    fn start_resetting(&mut self) {
        let r = &mut self.registers.borrow_mut().operational.port_registers;
        r.update(self.slot - 1, |r| r.port_sc.set_port_reset(true))
    }

    fn wait_until_reset_is_completed(&self) {
        let r = &self.registers.borrow().operational.port_registers;
        while !r.read(self.slot - 1).port_sc.port_reset_changed() {}
    }
}
