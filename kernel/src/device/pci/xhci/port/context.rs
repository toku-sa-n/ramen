// SPDX-License-Identifier: GPL-3.0-or-later

use crate::device::pci::xhci::structures::{
    context::{Context, EndpointType},
    ring::{transfer, CycleBit},
};

pub struct Initializer<'a> {
    context: &'a mut Context,
    ring: &'a transfer::Ring,
    port_id: u8,
}
impl<'a> Initializer<'a> {
    pub fn new(context: &'a mut Context, ring: &'a transfer::Ring, port_id: u8) -> Self {
        Self {
            context,
            ring,
            port_id,
        }
    }

    pub fn init(&mut self) {
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
        ep_0.set_dequeue_cycle_state(CycleBit::new(true));
        ep_0.set_error_count(3);
    }
}
