// SPDX-License-Identifier: GPL-3.0-or-later

use super::{descriptor_fetcher::DescriptorFetcher, endpoint, slot_assigned::SlotAssigned};
use crate::device::pci::xhci::{
    exchanger,
    exchanger::transfer,
    structures::{context::Context, descriptor::Descriptor},
};
use alloc::{sync::Arc, vec::Vec};
use spinning_top::Spinlock;
use transfer::DoorbellWriter;

pub(super) struct EndpointsInitializer {
    cx: Arc<Spinlock<Context>>,
    descriptors: Vec<Descriptor>,
    endpoints: Vec<endpoint::NonDefault>,
    slot_number: u8,
}
impl EndpointsInitializer {
    pub(super) fn new(f: DescriptorFetcher, descriptors: Vec<Descriptor>) -> Self {
        let cx = f.context();
        let endpoints = descriptors_to_endpoints(&f, &descriptors);
        let slot_number = f.slot_number();

        Self {
            cx,
            descriptors,
            endpoints,
            slot_number,
        }
    }

    pub(super) async fn init(mut self) -> SlotAssigned {
        self.init_contexts();
        self.configure_endpoint().await;
        SlotAssigned::new(self)
    }

    pub(super) fn slot_number(&self) -> u8 {
        self.slot_number
    }

    pub(super) fn context(&self) -> Arc<Spinlock<Context>> {
        self.cx.clone()
    }

    pub(super) fn descriptors(&self) -> Vec<Descriptor> {
        self.descriptors.clone()
    }

    pub(super) fn endpoints(self) -> Vec<endpoint::NonDefault> {
        self.endpoints
    }

    fn init_contexts(&mut self) {
        for e in &mut self.endpoints {
            e.init_context();
        }
    }

    async fn configure_endpoint(&mut self) {
        let a = self.cx.lock().input.phys_addr();
        exchanger::command::configure_endpoint(a, self.slot_number).await;
    }
}

fn descriptors_to_endpoints(
    f: &DescriptorFetcher,
    descriptors: &[Descriptor],
) -> Vec<endpoint::NonDefault> {
    descriptors
        .clone()
        .iter()
        .filter_map(|desc| {
            if let Descriptor::Endpoint(e) = desc {
                Some(endpoint::NonDefault::new(
                    *e,
                    f.context(),
                    transfer::Sender::new(DoorbellWriter::new(f.slot_number(), e.doorbell_value())),
                ))
            } else {
                None
            }
        })
        .collect()
}
