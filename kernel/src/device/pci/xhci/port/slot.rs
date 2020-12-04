// SPDX-License-Identifier: GPL-3.0-or-later

use super::{super::structures::descriptor::Descriptor, endpoint, Port};
use crate::{
    device::pci::xhci::{
        exchanger::{command, receiver::Receiver, transfer},
        structures::{context::Context, dcbaa::DeviceContextBaseAddressArray, descriptor},
    },
    mem::allocator::page_box::PageBox,
    Futurelock,
};
use alloc::{sync::Arc, vec::Vec};
use bit_field::BitField;
use endpoint::Endpoint;
use spinning_top::Spinlock;
use transfer::DoorbellWriter;

pub struct Slot {
    pub id: u8,
    dcbaa: Arc<Spinlock<DeviceContextBaseAddressArray>>,
    pub cx: Arc<Spinlock<Context>>,
    def_ep: endpoint::Default,
    recv: Arc<Spinlock<Receiver>>,
}
impl Slot {
    pub fn new(port: Port, id: u8, recv: Arc<Spinlock<Receiver>>) -> Self {
        let cx = Arc::new(Spinlock::new(port.context));
        let dbl_writer = DoorbellWriter::new(id, 1);
        Self {
            id,
            dcbaa: port.dcbaa,
            cx: cx.clone(),
            def_ep: endpoint::Default::new(transfer::Sender::new(recv.clone(), dbl_writer), cx),
            recv,
        }
    }

    pub async fn init(&mut self, runner: Arc<Futurelock<command::Sender>>) {
        self.init_default_ep();
        self.register_with_dcbaa();
        self.issue_address_device(runner).await;
    }

    pub async fn get_device_descriptor(&mut self) -> PageBox<descriptor::Device> {
        self.def_ep.get_device_descriptor().await
    }

    pub async fn endpoints(&mut self) -> Vec<Endpoint> {
        let ds = self.get_configuration_descriptors().await;
        let mut eps = Vec::new();

        for d in ds {
            if let Descriptor::Endpoint(ep) = d {
                eps.push(Endpoint::new(
                    ep,
                    self.cx.clone(),
                    transfer::Sender::new(
                        self.recv.clone(),
                        DoorbellWriter::new(
                            self.id,
                            2 * u32::from(ep.endpoint_address.get_bits(0..=3))
                                + ep.endpoint_address.get_bit(7) as u32,
                        ),
                    ),
                ));
            }
        }

        eps
    }

    pub async fn get_configuration_descriptors(&mut self) -> Vec<Descriptor> {
        let r = self.get_raw_configuration_descriptors().await;
        RawDescriptorParser::new(r).parse()
    }

    fn init_default_ep(&mut self) {
        self.def_ep.init_context();
    }

    async fn get_raw_configuration_descriptors(&mut self) -> PageBox<[u8]> {
        self.def_ep.get_raw_configuration_descriptors().await
    }

    fn register_with_dcbaa(&mut self) {
        self.dcbaa.lock()[self.id.into()] = self.cx.lock().output_device.phys_addr();
    }

    async fn issue_address_device(&mut self, runner: Arc<Futurelock<command::Sender>>) {
        let cx_addr = self.cx.lock().input.phys_addr();
        runner.lock().await.address_device(cx_addr, self.id).await;
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
                Err(e) => warn!("Error: {:?}", e),
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
