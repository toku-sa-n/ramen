// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
    exchanger::command::Sender,
    structures::{
        context::{Context, EndpointType},
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
        ContextInitializer::new(
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

struct ContextInitializer<'a> {
    context: &'a mut Context,
    ring: &'a transfer::Ring,
    port_id: u8,
}
impl<'a> ContextInitializer<'a> {
    fn new(context: &'a mut Context, ring: &'a transfer::Ring, port_id: u8) -> Self {
        Self {
            context,
            ring,
            port_id,
        }
    }

    fn init(&mut self) {
        self.init_input_control();
        self.init_input_slot();
        self.init_input_default_control_endpoint0();
    }

    fn init_input_control(&mut self) {
        let input_control = self.context.input.control_mut();
        input_control.set_aflag(0);
        input_control.set_aflag(1);
    }

    fn init_input_slot(&mut self) {
        let slot = &mut self.context.input.device_mut().slot;
        slot.set_context_entries(1);
        slot.set_root_hub_port_number(self.port_id);
    }

    fn init_input_default_control_endpoint0(&mut self) {
        let ep_0 = &mut self.context.input.device_mut().ep_0;
        ep_0.set_endpoint_type(EndpointType::Control);

        // FIXME: Support other sppeds.
        ep_0.set_max_packet_size(64);
        ep_0.set_dequeue_ptr(self.ring.phys_addr());
        ep_0.set_dequeue_cycle_state(true);
        ep_0.set_error_count(3);
    }
}
