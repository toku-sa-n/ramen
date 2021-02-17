// SPDX-License-Identifier: GPL-3.0-or-later

use super::{endpoint, endpoints_initializer::EndpointsInitializer};
use crate::device::pci::xhci::structures::{context::Context, descriptor};
use alloc::{sync::Arc, vec::Vec};
use descriptor::Descriptor;
use spinning_top::Spinlock;

pub(super) struct SlotAssigned {
    slot_number: u8,
    cx: Arc<Spinlock<Context>>,
    descriptors: Vec<Descriptor>,
    endpoints: Vec<endpoint::NonDefault>,
}
impl SlotAssigned {
    pub(super) fn new(i: EndpointsInitializer) -> Self {
        let slot_number = i.slot_number();
        let cx = i.context();
        let descriptors = i.descriptors();
        let endpoints = i.endpoints();

        Self {
            slot_number,
            cx,
            descriptors,
            endpoints,
        }
    }

    pub(super) fn context(&self) -> Arc<Spinlock<Context>> {
        self.cx.clone()
    }

    pub(super) fn descriptors(&self) -> Vec<Descriptor> {
        self.descriptors.clone()
    }

    pub(super) fn slot_number(&self) -> u8 {
        self.slot_number
    }

    pub(super) fn endpoints(self) -> Vec<endpoint::NonDefault> {
        self.endpoints
    }
}
