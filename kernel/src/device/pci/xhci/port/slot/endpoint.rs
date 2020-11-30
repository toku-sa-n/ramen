// SPDX-License-Identifier: GPL-3.0-or-later

use super::Slot;
use crate::{
    device::pci::xhci::{
        exchanger::{command, transfer},
        structures::{
            context::{self, Context},
            descriptor,
            ring::CycleBit,
        },
    },
    mem::allocator::page_box::PageBox,
};
use alloc::{rc::Rc, vec::Vec};
use bit_field::BitField;
use context::EndpointType;
use core::cell::RefCell;
use futures_intrusive::sync::LocalMutex;
use num_traits::FromPrimitive;

pub struct Collection {
    def: Default,
    eps: Vec<Endpoint>,
    cx: Rc<RefCell<Context>>,
    cmd: Rc<LocalMutex<command::Sender>>,
    slot_id: u8,
}
impl Collection {
    pub async fn new(mut slot: Slot, cmd: Rc<LocalMutex<command::Sender>>) -> Self {
        let eps = slot.endpoints().await;
        info!("Endpoints collected");
        Self {
            def: slot.def_ep,
            eps,
            cx: slot.cx,
            cmd,
            slot_id: slot.id,
        }
    }

    pub async fn init(&mut self) {
        self.enable_eps();
        self.issue_configure_eps().await;
        info!("Endpoints initialized");
    }

    fn enable_eps(&mut self) {
        for ep in &mut self.eps {
            ep.init_context();
        }
    }

    async fn issue_configure_eps(&mut self) {
        let mut cmd = self.cmd.lock().await;
        let cx_addr = self.cx.borrow().input.phys_addr();
        cmd.configure_endpoint(cx_addr, self.slot_id).await;
    }
}

pub struct Endpoint {
    desc: descriptor::Endpoint,
    cx: Rc<RefCell<Context>>,
    sender: transfer::Sender,
}
impl Endpoint {
    pub fn new(
        desc: descriptor::Endpoint,
        cx: Rc<RefCell<Context>>,
        sender: transfer::Sender,
    ) -> Self {
        Self { desc, cx, sender }
    }

    pub fn init_context(&mut self) {
        ContextInitializer::new(&self.desc, &mut self.cx.borrow_mut(), &self.sender).init();
    }
}

pub struct Default {
    sender: transfer::Sender,
    cx: Rc<RefCell<Context>>,
}
impl Default {
    pub fn new(sender: transfer::Sender, cx: Rc<RefCell<Context>>) -> Self {
        Self { sender, cx }
    }

    pub async fn get_device_descriptor(&mut self) -> PageBox<descriptor::Device> {
        self.sender.get_device_descriptor().await
    }

    pub async fn get_raw_configuration_descriptors(&mut self) -> PageBox<[u8]> {
        self.sender.get_configuration_descriptor().await
    }

    pub fn init_context(&mut self) {
        let mut cx = self.cx.borrow_mut();
        let ep_0 = &mut cx.input.device_mut().ep_0;
        ep_0.set_endpoint_type(EndpointType::Control);

        // TODO: Support other speeds.
        ep_0.set_max_packet_size(64);
        ep_0.set_dequeue_ptr(self.sender.ring_addr());
        ep_0.set_dequeue_cycle_state(CycleBit::new(true));
        ep_0.set_error_count(3);
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

        self.context.input.control_mut().clear_aflag(1); // See xHCI dev manual 4.6.6.
        self.context.input.control_mut().set_aflag(dci);
    }

    fn calculate_dci(&self) -> u8 {
        let a = self.ep.endpoint_address;
        2 * a.get_bits(0..=3) + a.get_bit(7) as u8
    }

    fn init_ep_context(&mut self) {
        let ep_ty = self.ep_ty();
        let max_packet_size = self.ep.max_packet_size;
        let interval = self.ep.interval;
        let ring_addr = self.sender.ring_addr();

        let c = self.ep_context();
        c.set_endpoint_type(ep_ty);
        c.set_max_packet_size(max_packet_size);
        c.set_max_burst_size(0);
        c.set_dequeue_cycle_state(CycleBit::new(true));
        c.set_max_primary_streams(0);
        c.set_mult(0);
        c.set_error_count(3);
        c.set_interval(interval);
        c.set_dequeue_ptr(ring_addr);
    }

    fn ep_context(&mut self) -> &mut context::Endpoint {
        let ep_idx: usize = self.ep.endpoint_address.get_bits(0..=3).into();
        let out_input = self.ep.endpoint_address.get_bit(7);
        let context_inout = &mut self.context.output_device.ep_inout[ep_idx];
        if out_input {
            &mut context_inout.input
        } else {
            &mut context_inout.out
        }
    }

    fn ep_ty(&self) -> context::EndpointType {
        context::EndpointType::from_u8(if self.ep.attributes == 0 {
            0
        } else {
            self.ep.attributes + if self.ep.endpoint_address == 0 { 0 } else { 4 }
        })
        .unwrap()
    }
}
