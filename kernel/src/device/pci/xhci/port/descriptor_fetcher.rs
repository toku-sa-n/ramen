// SPDX-License-Identifier: GPL-3.0-or-later

use super::{endpoint, max_packet_size_setter::MaxPacketSizeSetter, slot_assigned::SlotAssigned};
use crate::device::pci::xhci::structures::{context::Context, descriptor, descriptor::Descriptor};
use alloc::{sync::Arc, vec::Vec};
use page_box::PageBox;
use spinning_top::Spinlock;

pub(super) struct DescriptorFetcher {
    slot_number: u8,
    cx: Arc<Spinlock<Context>>,
    ep0: endpoint::Default,
}
impl DescriptorFetcher {
    pub(super) fn new(s: MaxPacketSizeSetter) -> Self {
        let slot_number = s.slot_number();
        let cx = s.context();
        let ep0 = s.ep0();

        Self {
            slot_number,
            cx,
            ep0,
        }
    }

    pub(super) async fn fetch(mut self) -> SlotAssigned {
        let r = self.get_raw_descriptors().await;
        let ds = RawDescriptorParser::new(r).parse();
        SlotAssigned::new(self, ds)
    }

    pub(super) fn context(&self) -> Arc<Spinlock<Context>> {
        self.cx.clone()
    }

    pub(super) fn slot_number(&self) -> u8 {
        self.slot_number
    }

    async fn get_raw_descriptors(&mut self) -> PageBox<[u8]> {
        self.ep0.get_raw_configuration_descriptors().await
    }
}

struct RawDescriptorParser {
    raw: PageBox<[u8]>,
    current: usize,
    len: usize,
}
impl RawDescriptorParser {
    fn new(raw: PageBox<[u8]>) -> Self {
        let len = raw.len();

        Self {
            raw,
            current: 0,
            len,
        }
    }

    fn parse(&mut self) -> Vec<Descriptor> {
        let mut v = Vec::new();
        while self.current < self.len && self.raw[self.current] > 0 {
            match self.parse_first_descriptor() {
                Ok(t) => v.push(t),
                Err(e) => debug!("Unrecognized USB descriptor: {:?}", e),
            }
        }
        v
    }

    fn parse_first_descriptor(&mut self) -> Result<Descriptor, descriptor::Error> {
        let raw = self.cut_raw_descriptor();
        Descriptor::from_slice(&raw)
    }

    fn cut_raw_descriptor(&mut self) -> Vec<u8> {
        let len: usize = self.raw[self.current].into();
        let v = self.raw[self.current..(self.current + len)].to_vec();
        self.current += len;
        v
    }
}
