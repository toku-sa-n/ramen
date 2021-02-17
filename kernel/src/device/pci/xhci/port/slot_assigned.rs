// SPDX-License-Identifier: GPL-3.0-or-later

use super::{endpoint, max_packet_size_setter::MaxPacketSizeSetter};
use crate::device::pci::xhci::{
    exchanger,
    structures::{context::Context, descriptor},
};
use alloc::{sync::Arc, vec::Vec};
use descriptor::Descriptor;
use exchanger::{transfer, transfer::DoorbellWriter};
use page_box::PageBox;
use spinning_top::Spinlock;

pub(super) struct SlotAssigned {
    slot_number: u8,
    cx: Arc<Spinlock<Context>>,
    def_ep: endpoint::Default,
}
impl SlotAssigned {
    pub(super) async fn new(s: MaxPacketSizeSetter) -> Self {
        let slot_number = s.slot_number();
        let cx = s.context();
        let ep = s.ep0();

        Self {
            slot_number,
            cx,
            def_ep: ep,
        }
    }

    pub(super) fn context(&self) -> Arc<Spinlock<Context>> {
        self.cx.clone()
    }

    pub fn id(&self) -> u8 {
        self.slot_number
    }

    pub(super) async fn get_device_descriptor(&mut self) -> PageBox<descriptor::Device> {
        self.def_ep.get_device_descriptor().await
    }

    pub async fn endpoints(&mut self) -> Vec<endpoint::NonDefault> {
        let ds = self.get_configuration_descriptors().await;
        let mut eps = Vec::new();

        for d in ds {
            if let Descriptor::Endpoint(ep) = d {
                eps.push(self.generate_endpoint(ep));
            }
        }

        eps
    }

    pub async fn interface_descriptor(&mut self) -> descriptor::Interface {
        *self
            .get_configuration_descriptors()
            .await
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

    pub async fn get_configuration_descriptors(&mut self) -> Vec<Descriptor> {
        let r = self.get_raw_configuration_descriptors().await;
        RawDescriptorParser::new(r).parse()
    }

    fn generate_endpoint(&self, ep: descriptor::Endpoint) -> endpoint::NonDefault {
        endpoint::NonDefault::new(
            ep,
            self.cx.clone(),
            transfer::Sender::new(DoorbellWriter::new(self.slot_number, ep.doorbell_value())),
        )
    }

    async fn get_raw_configuration_descriptors(&mut self) -> PageBox<[u8]> {
        self.def_ep.get_raw_configuration_descriptors().await
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