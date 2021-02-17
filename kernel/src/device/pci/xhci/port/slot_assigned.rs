// SPDX-License-Identifier: GPL-3.0-or-later

use super::{descriptor_fetcher::DescriptorFetcher, endpoint};
use crate::device::pci::xhci::{
    exchanger,
    structures::{context::Context, descriptor},
};
use alloc::{sync::Arc, vec::Vec};
use descriptor::Descriptor;
use exchanger::{transfer, transfer::DoorbellWriter};
use spinning_top::Spinlock;

pub(super) struct SlotAssigned {
    slot_number: u8,
    cx: Arc<Spinlock<Context>>,
    descriptors: Vec<Descriptor>,
}
impl SlotAssigned {
    pub(super) fn new(f: DescriptorFetcher, descriptors: Vec<Descriptor>) -> Self {
        let slot_number = f.slot_number();
        let cx = f.context();

        Self {
            slot_number,
            cx,
            descriptors,
        }
    }

    pub(super) fn context(&self) -> Arc<Spinlock<Context>> {
        self.cx.clone()
    }

    pub fn id(&self) -> u8 {
        self.slot_number
    }

    pub async fn endpoints(&mut self) -> Vec<endpoint::NonDefault> {
        let mut eps = Vec::new();

        for d in &self.descriptors {
            if let Descriptor::Endpoint(ep) = d {
                eps.push(self.generate_endpoint(ep.clone()));
            }
        }

        eps
    }

    pub async fn interface_descriptor(&mut self) -> descriptor::Interface {
        *self
            .descriptors
            .iter()
            .find_map(|x| {
                if let Descriptor::Interface(e) = x {
                    Some(e)
                } else {
                    None
                }
            })
            .unwrap()
    }

    fn generate_endpoint(&self, ep: descriptor::Endpoint) -> endpoint::NonDefault {
        endpoint::NonDefault::new(
            ep,
            self.cx.clone(),
            transfer::Sender::new(DoorbellWriter::new(self.slot_number, ep.doorbell_value())),
        )
    }
}
