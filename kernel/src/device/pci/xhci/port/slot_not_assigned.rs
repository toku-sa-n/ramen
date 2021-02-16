// SPDX-License-Identifier: GPL-3.0-or-later

use super::{context, resetter::Resetter};
use crate::device::pci::xhci::structures::{context::Context, registers};
use alloc::sync::Arc;
use spinning_top::Spinlock;
use xhci::registers::PortRegisterSet;

pub(super) struct SlotNotAssigned {
    port_number: u8,
    context: Arc<Spinlock<Context>>,
}
impl SlotNotAssigned {
    pub(super) fn new(port_number: u8) -> Self {
        Self {
            port_number,
            context: Arc::new(Spinlock::new(Context::default())),
        }
    }

    pub(super) fn port_number(&self) -> u8 {
        self.port_number
    }

    pub(super) fn context(&self) -> Arc<Spinlock<Context>> {
        self.context.clone()
    }

    pub(super) fn connected(&self) -> bool {
        self.read_port_rg().portsc.current_connect_status()
    }

    pub(super) fn reset(&mut self) {
        info!("Resetting port {}", self.port_number);
        Resetter::new(self.port_number).reset();
        info!("Port {} is reset.", self.port_number);
    }

    pub(super) fn init_context(&mut self) {
        context::Initializer::new(&mut self.context.lock(), self.port_number).init();
    }

    fn read_port_rg(&self) -> PortRegisterSet {
        registers::handle(|r| r.port_register_set.read_at((self.port_number - 1).into()))
    }
}
