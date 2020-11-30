// SPDX-License-Identifier: GPL-3.0-or-later

use crate::device::pci::xhci::structures::context::Context;

pub struct Initializer<'a> {
    context: &'a mut Context,
    port_id: u8,
}
impl<'a> Initializer<'a> {
    pub fn new(context: &'a mut Context, port_id: u8) -> Self {
        Self { context, port_id }
    }

    pub fn init(&mut self) {
        self.init_input_control();
        self.init_input_slot();
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
}
