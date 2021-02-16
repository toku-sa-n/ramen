// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
    exchanger,
    structures::{context::Context, dcbaa, descriptor, registers},
};
use crate::multitask::{self, task::Task};
use alloc::{collections::VecDeque, sync::Arc, vec::Vec};
use conquer_once::spin::Lazy;
use core::{future::Future, pin::Pin, task::Poll};
use descriptor::Descriptor;
use endpoint::Endpoint;
use exchanger::{transfer, transfer::DoorbellWriter};
use futures_util::task::AtomicWaker;
use page_box::PageBox;
use slot_not_assigned::SlotNotAssigned;
use spinning_top::Spinlock;

mod class_driver;
mod endpoint;
mod resetter;
mod slot_not_assigned;
mod spawner;

static CURRENT_RESET_PORT: Lazy<Spinlock<ResetPort>> =
    Lazy::new(|| Spinlock::new(ResetPort::new()));

struct ResetPort {
    resetting: bool,
    wakers: VecDeque<AtomicWaker>,
}
impl ResetPort {
    fn new() -> Self {
        Self {
            resetting: false,
            wakers: VecDeque::new(),
        }
    }

    fn complete_reset(&mut self) {
        self.resetting = false;
        if let Some(w) = self.wakers.pop_front() {
            w.wake();
        }
    }

    fn resettable(&mut self, waker: AtomicWaker) -> bool {
        if self.resetting {
            self.wakers.push_back(waker);
            false
        } else {
            self.resetting = true;
            true
        }
    }
}

pub fn try_spawn(port_idx: u8) -> Result<(), spawner::PortNotConnected> {
    spawner::try_spawn(port_idx)
}

async fn main(port: SlotNotAssigned) {
    let mut eps = init_port_and_slot_exclusively(port).await;
    eps.init().await;

    match eps.ty() {
        (3, 1, 2) => {
            multitask::add(Task::new_poll(class_driver::mouse::task(eps)));
        }
        (3, 1, 1) => {
            multitask::add(Task::new_poll(class_driver::keyboard::task(eps)));
        }
        (8, _, _) => multitask::add(Task::new(class_driver::mass_storage::task(eps))),
        t => warn!("Unknown device: {:?}", t),
    }
}

async fn init_port_and_slot_exclusively(port: SlotNotAssigned) -> endpoint::AddressAssigned {
    let reset_waiter = ResetWaiterFuture;
    reset_waiter.await;

    let port_idx = port.port_number();
    let slot = init_port_and_slot(port).await;
    CURRENT_RESET_PORT.lock().complete_reset();
    info!("Port {} reset completed.", port_idx);
    endpoint::AddressAssigned::new(slot).await
}

async fn init_port_and_slot(mut p: SlotNotAssigned) -> SlotAssigned {
    p.reset();
    p.init_context();

    let mut slot = SlotAssigned::new(p).await;
    slot.init().await;
    debug!("Slot initialized");
    slot
}

pub fn spawn_all_connected_port_tasks() {
    spawner::spawn_all_connected_ports();
}

fn max_num() -> u8 {
    registers::handle(|r| r.capability.hcsparams1.read().number_of_ports())
}

pub struct SlotAssigned {
    slot_number: u8,
    cx: Arc<Spinlock<Context>>,
    def_ep: endpoint::Default,
}
impl SlotAssigned {
    async fn new(port: SlotNotAssigned) -> Self {
        let slot_number = exchanger::command::enable_device_slot().await;
        let cx = port.context();
        let dbl_writer = DoorbellWriter::new(slot_number, 1);
        Self {
            slot_number,
            cx: cx.clone(),
            def_ep: endpoint::Default::new(
                transfer::Sender::new(dbl_writer),
                cx,
                port.port_number(),
            ),
        }
    }

    pub(in crate::device::pci::xhci) fn context(&self) -> Arc<Spinlock<Context>> {
        self.cx.clone()
    }

    pub fn id(&self) -> u8 {
        self.slot_number
    }

    pub async fn init(&mut self) {
        self.init_default_ep();
        self.register_with_dcbaa();
        self.issue_address_device().await;
    }

    pub async fn get_device_descriptor(&mut self) -> PageBox<descriptor::Device> {
        self.def_ep.get_device_descriptor().await
    }

    pub async fn endpoints(&mut self) -> Vec<Endpoint> {
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

    fn generate_endpoint(&self, ep: descriptor::Endpoint) -> Endpoint {
        Endpoint::new(
            ep,
            self.cx.clone(),
            transfer::Sender::new(DoorbellWriter::new(self.slot_number, ep.doorbell_value())),
        )
    }

    fn init_default_ep(&mut self) {
        self.def_ep.init_context();
    }

    async fn get_raw_configuration_descriptors(&mut self) -> PageBox<[u8]> {
        self.def_ep.get_raw_configuration_descriptors().await
    }

    fn register_with_dcbaa(&mut self) {
        let a = self.cx.lock().output.phys_addr();
        dcbaa::register(self.slot_number.into(), a);
    }

    async fn issue_address_device(&mut self) {
        let cx_addr = self.cx.lock().input.phys_addr();
        exchanger::command::address_device(cx_addr, self.slot_number).await;
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

struct ResetWaiterFuture;
impl Future for ResetWaiterFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
        let waker = AtomicWaker::new();
        waker.register(cx.waker());
        if CURRENT_RESET_PORT.lock().resettable(waker) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
