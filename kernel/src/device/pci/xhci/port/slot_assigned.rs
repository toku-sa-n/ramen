// SPDX-License-Identifier: GPL-3.0-or-later

use super::{endpoint, endpoints_initializer::EndpointsInitializer};
use crate::device::pci::xhci::structures::descriptor;
use alloc::vec::Vec;
use descriptor::Descriptor;

pub(super) struct SlotAssigned {
    descriptors: Vec<Descriptor>,
    endpoints: Vec<endpoint::NonDefault>,
}
impl SlotAssigned {
    pub(super) fn new(i: EndpointsInitializer) -> Self {
        let descriptors = i.descriptors();
        let endpoints = i.endpoints();

        Self {
            descriptors,
            endpoints,
        }
    }

    pub(super) fn descriptors(&self) -> Vec<Descriptor> {
        self.descriptors.clone()
    }

    pub(super) fn endpoints(self) -> Vec<endpoint::NonDefault> {
        self.endpoints
    }
}
