// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{
        command_runner::Runner,
        context::{self, EndpointType},
        dcbaa::DeviceContextBaseAddressArray,
        register::{hc_operational::PortRegisters, Registers},
        ring::transfer,
    },
    crate::{
        mem::allocator::page_box::PageBox,
        multitask::task::{self, Task},
    },
    alloc::rc::Rc,
    core::cell::RefCell,
    futures_intrusive::sync::LocalMutex,
};

async fn task(mut port: Port, command_runner: Rc<LocalMutex<Runner>>) {
    port.reset_if_connected();

    let slot_id = command_runner
        .lock()
        .await
        .enable_device_slot()
        .await
        .unwrap();

    port.init_input_context();
    port.init_input_slot_context();
    port.init_input_default_control_endpoint0_context();
    port.register_to_dcbaa(slot_id.into());
}

// FIXME: Resolve this.
#[allow(clippy::too_many_arguments)]
pub fn spawn_tasks(
    command_runner: &Rc<LocalMutex<Runner>>,
    dcbaa: &Rc<RefCell<DeviceContextBaseAddressArray>>,
    registers: &Rc<RefCell<Registers>>,
    task_collection: &Rc<RefCell<task::Collection>>,
) {
    for i in 0..num_of_ports(&registers) {
        let port = Port::new(registers.clone(), dcbaa.clone(), i);
        if port.connected() {
            task_collection
                .borrow_mut()
                .add_task_as_woken(Task::new(task(port, command_runner.clone())));
        }
    }
}

fn num_of_ports(registers: &Rc<RefCell<Registers>>) -> usize {
    let params1 = registers.borrow().hc_capability.hcs_params_1.read();
    params1.max_ports().into()
}

pub struct Port {
    registers: Rc<RefCell<Registers>>,
    index: usize,
    input_context: PageBox<context::Input>,
    input_slot_context: PageBox<context::Slot>,
    output_device_context: PageBox<context::Device>,
    transfer_ring: transfer::Ring,
    dcbaa: Rc<RefCell<DeviceContextBaseAddressArray>>,
}
impl Port {
    fn reset_if_connected(&mut self) {
        if self.connected() {
            self.reset();
        }
    }

    fn new(
        registers: Rc<RefCell<Registers>>,
        dcbaa: Rc<RefCell<DeviceContextBaseAddressArray>>,
        index: usize,
    ) -> Self {
        Self {
            registers: registers.clone(),
            index,
            input_context: PageBox::new(context::Input::null()),
            input_slot_context: PageBox::new(context::Slot::null()),
            output_device_context: PageBox::new(context::Device::null()),
            dcbaa,
            transfer_ring: transfer::Ring::new(registers),
        }
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

    fn init_input_context(&mut self) {
        self.input_context.input_control.set_aflag(0);
        self.input_context.input_control.set_aflag(1);
    }

    fn init_input_slot_context(&mut self) {
        self.input_slot_context.set_context_entries(1);
    }

    fn init_input_default_control_endpoint0_context(&mut self) {
        let ep_0 = &mut self.input_context.device.ep_0.0;
        ep_0.set_endpoint_type(EndpointType::Control);
        ep_0.set_dequeue_ptr(self.transfer_ring.phys_addr().as_u64());
        ep_0.set_dequeue_cycle_state(false);
        ep_0.set_error_count(3);
    }

    fn register_to_dcbaa(&mut self, slot_id: usize) {
        self.dcbaa.borrow_mut()[slot_id] = self.output_device_context.phys_addr();
    }

    fn read_port_rg(&self) -> PortRegisters {
        let port_rg = &self.registers.borrow().hc_operational.port_registers;
        port_rg.read(self.index)
    }
}
