// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
    endpoint, slot_assigned::SlotAssigned, slot_structures_initializer::SlotStructuresInitializer,
};
use crate::device::pci::xhci::structures::context::Context;
use alloc::sync::Arc;
use spinning_top::Spinlock;

pub(super) struct MaxPacketSizeSetter {
    ep: endpoint::Default,
    cx: Arc<Spinlock<Context>>,
    slot_number: u8,
}
impl MaxPacketSizeSetter {
    pub(super) fn new(i: SlotStructuresInitializer) -> Self {
        let cx = i.context();
        let slot_number = i.slot_number();
        let ep = i.ep0();

        Self {
            ep,
            cx,
            slot_number,
        }
    }

    pub(super) async fn set(mut self) -> SlotAssigned {
        let s = self.max_packet_size().await;
        self.set_max_packet_size(s);
        SlotAssigned::new(self).await
    }

    pub(super) fn slot_number(&self) -> u8 {
        self.slot_number
    }

    pub(super) fn context(&self) -> Arc<Spinlock<Context>> {
        self.cx.clone()
    }

    pub(super) fn ep0(self) -> endpoint::Default {
        self.ep
    }

    async fn max_packet_size(&mut self) -> u16 {
        self.ep.get_max_packet_size().await
    }

    fn set_max_packet_size(&mut self, s: u16) {
        let mut cx = self.cx.lock();
        let ep_0 = cx.input.device_mut().endpoint0_mut();

        ep_0.set_max_packet_size(s);
    }
}
