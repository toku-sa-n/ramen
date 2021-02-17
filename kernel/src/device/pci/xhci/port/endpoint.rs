// SPDX-License-Identifier: GPL-3.0-or-later

use super::SlotAssigned;
use crate::device::pci::xhci::{
    exchanger::{self, transfer},
    structures::{context::Context, descriptor},
};
use alloc::{sync::Arc, vec::Vec};
use bit_field::BitField;
use core::slice;
use page_box::PageBox;
use spinning_top::Spinlock;
use x86_64::PhysAddr;
use xhci::context::{EndpointHandler, EndpointType};

pub struct AddressAssigned {
    eps: Vec<NonDefault>,
    cx: Arc<Spinlock<Context>>,
    interface: descriptor::Interface,
    slot_id: u8,
}
impl AddressAssigned {
    pub(super) async fn new(mut slot: SlotAssigned) -> Self {
        let eps = slot.endpoints().await;
        let interface = slot.interface_descriptor().await;
        debug!("Endpoints collected");
        Self {
            eps,
            cx: slot.context(),
            interface,
            slot_id: slot.id(),
        }
    }

    pub async fn init(&mut self) {
        self.init_ep_contexts();
        self.issue_configure_eps().await;
        debug!("Endpoints initialized");
    }

    pub fn ty(&self) -> (u8, u8, u8) {
        self.interface.ty()
    }

    pub(in crate::device::pci::xhci) async fn issue_normal_trb<T>(
        &mut self,
        b: &PageBox<T>,
        ty: EndpointType,
    ) -> Result<(), Error> {
        for ep in &mut self.eps {
            if ep.ty() == ty {
                ep.issue_normal_trb(b).await;
                return Ok(());
            }
        }

        Err(Error::NoSuchEndpoint(ty))
    }

    fn init_ep_contexts(&mut self) {
        for ep in &mut self.eps {
            ep.init_context();
        }
    }

    async fn issue_configure_eps(&mut self) {
        let cx_addr = self.cx.lock().input.phys_addr();
        exchanger::command::configure_endpoint(cx_addr, self.slot_id).await;
    }
}
impl<'a> IntoIterator for &'a mut AddressAssigned {
    type Item = &'a mut NonDefault;
    type IntoIter = slice::IterMut<'a, NonDefault>;

    fn into_iter(self) -> Self::IntoIter {
        self.eps.iter_mut()
    }
}

pub struct NonDefault {
    desc: descriptor::Endpoint,
    cx: Arc<Spinlock<Context>>,
    sender: transfer::Sender,
}
impl NonDefault {
    pub(in crate::device::pci::xhci) fn new(
        desc: descriptor::Endpoint,
        cx: Arc<Spinlock<Context>>,
        sender: transfer::Sender,
    ) -> Self {
        Self { desc, cx, sender }
    }

    pub fn init_context(&mut self) {
        ContextInitializer::new(&self.desc, &mut self.cx.lock(), &self.sender).init();
    }

    pub fn ty(&self) -> EndpointType {
        self.desc.ty()
    }

    async fn issue_normal_trb<T: ?Sized>(&mut self, b: &PageBox<T>) {
        self.sender.issue_normal_trb(b).await
    }
}

pub struct Default {
    sender: transfer::Sender,
}
impl Default {
    pub(in crate::device::pci::xhci) fn new(sender: transfer::Sender) -> Self {
        Self { sender }
    }

    pub(super) fn ring_addr(&self) -> PhysAddr {
        self.sender.ring_addr()
    }

    pub(super) async fn get_device_descriptor(&mut self) -> PageBox<descriptor::Device> {
        self.sender.get_device_descriptor().await
    }

    pub(super) async fn get_max_packet_size(&mut self) -> u16 {
        self.sender
            .get_max_packet_size_from_device_descriptor()
            .await
    }

    pub(super) async fn get_raw_configuration_descriptors(&mut self) -> PageBox<[u8]> {
        self.sender.get_configuration_descriptor().await
    }
}

struct ContextInitializer<'a> {
    ep: &'a descriptor::Endpoint,
    context: &'a mut Context,
    sender: &'a transfer::Sender,
}
impl<'a> ContextInitializer<'a> {
    fn new(
        ep: &'a descriptor::Endpoint,
        context: &'a mut Context,
        sender: &'a transfer::Sender,
    ) -> Self {
        Self {
            ep,
            context,
            sender,
        }
    }

    fn init(&mut self) {
        self.set_aflag();
        self.init_ep_context();
    }

    fn set_aflag(&mut self) {
        let dci: usize = self.calculate_dci().into();
        let c = self.context.input.control_mut();

        c.clear_aflag(1); // See xHCI dev manual 4.6.6.
        c.set_aflag(dci);
    }

    fn calculate_dci(&self) -> u8 {
        let a = self.ep.endpoint_address;
        2 * a.get_bits(0..=3) + a.get_bit(7) as u8
    }

    fn init_ep_context(&mut self) {
        let ep_ty = self.ep.ty();
        let max_packet_size = self.ep.max_packet_size;
        let interval = self.ep.interval;
        let ring_addr = self.sender.ring_addr();

        debug!("Endpoint type: {:?}", ep_ty);

        let c = self.ep_context();
        c.set_endpoint_type(ep_ty);
        c.set_max_packet_size(max_packet_size);
        c.set_max_burst_size(0);
        c.set_dequeue_cycle_state(true);
        c.set_max_primary_streams(0);
        c.set_mult(0);
        c.set_error_count(3);
        c.set_interval(interval);
        c.set_transfer_ring_dequeue_pointer(ring_addr.as_u64());
    }

    fn ep_context(&mut self) -> &mut dyn EndpointHandler {
        let ep_i: usize = self.ep.endpoint_address.get_bits(0..=3).into();
        let is_input = self.ep.endpoint_address.get_bit(7);
        let context_inout = self.context.input.device_mut().endpoints_mut(ep_i);
        if is_input {
            context_inout.input_mut()
        } else {
            context_inout.output_mut()
        }
    }
}

#[derive(Debug)]
pub(in crate::device::pci::xhci) enum Error {
    NoSuchEndpoint(EndpointType),
}
