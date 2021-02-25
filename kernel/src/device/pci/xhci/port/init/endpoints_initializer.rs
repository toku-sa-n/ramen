// SPDX-License-Identifier: GPL-3.0-or-later

use super::{descriptor_fetcher::DescriptorFetcher, fully_operational::FullyOperational};
use crate::device::pci::xhci::{
    exchanger,
    exchanger::transfer,
    port::endpoint,
    structures::{context::Context, descriptor::Descriptor},
};
use alloc::{sync::Arc, vec::Vec};
use spinning_top::Spinlock;
use transfer::DoorbellWriter;

pub(super) struct EndpointsInitializer {
    cx: Arc<Spinlock<Context>>,
    descriptors: Vec<Descriptor>,
    endpoints: Vec<endpoint::NonDefault>,
    ep0: endpoint::Default,
    slot_number: u8,
}
impl EndpointsInitializer {
    #[allow(clippy::needless_pass_by_value)] // `DescriptorFetcher` should be consumed.
    pub(super) fn new(f: DescriptorFetcher, descriptors: Vec<Descriptor>) -> Self {
        let cx = f.context();
        let endpoints = descriptors_to_endpoints(&f, &descriptors);
        let slot_number = f.slot_number();
        let ep0 = f.ep0();

        Self {
            cx,
            descriptors,
            endpoints,
            ep0,
            slot_number,
        }
    }

    pub(super) async fn init(mut self) -> FullyOperational {
        self.init_contexts();
        self.set_context_entries();
        self.configure_endpoint().await;
        FullyOperational::new(self)
    }

    pub(super) fn descriptors(&self) -> Vec<Descriptor> {
        self.descriptors.clone()
    }

    pub(super) fn endpoints(self) -> (endpoint::Default, Vec<endpoint::NonDefault>) {
        (self.ep0, self.endpoints)
    }

    fn init_contexts(&mut self) {
        for e in &mut self.endpoints {
            e.init_context();
        }
    }

    fn set_context_entries(&mut self) {
        let mut cx = self.cx.lock();
        cx.input.device_mut().slot_mut().set_context_entries(31);
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
