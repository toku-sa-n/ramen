// SPDX-License-Identifier: GPL-3.0-or-later

use super::receiver::{self, ReceiveFuture};
use crate::device::pci::xhci::structures::{descriptor, registers, ring::transfer};
use alloc::{sync::Arc, vec::Vec};
use core::convert::TryInto;
use futures_util::task::AtomicWaker;
use page_box::PageBox;
use spinning_top::Spinlock;
use x86_64::PhysAddr;
use xhci::ring::trb::{
    event, transfer as transfer_trb,
    transfer::{Direction, Normal, TransferType},
};

pub(in crate::device::pci::xhci) struct Sender {
    channel: Channel,
}
impl Sender {
    pub(in crate::device::pci::xhci) fn new(doorbell_writer: DoorbellWriter) -> Self {
        Self {
            channel: Channel::new(doorbell_writer),
        }
    }

    pub(in crate::device::pci::xhci) fn ring_addr(&self) -> PhysAddr {
        self.channel.ring_addr()
    }

    pub(in crate::device::pci::xhci) async fn get_max_packet_size_from_device_descriptor(
        &mut self,
    ) -> u16 {
        let b = PageBox::from(descriptor::Device::default());

        let setup = *transfer_trb::SetupStage::default()
            .set_transfer_type(TransferType::In)
            .set_trb_transfer_length(8)
            .set_interrupt_on_completion(false)
            .set_request_type(0x80)
            .set_request(6)
            .set_value(0x0100)
            .set_length(8);

        let data = *transfer_trb::DataStage::default()
            .set_direction(Direction::In)
            .set_trb_transfer_length(8)
            .set_interrupt_on_completion(false)
            .set_data_buffer_pointer(b.phys_addr().as_u64());

        let status = *transfer_trb::StatusStage::default().set_interrupt_on_completion(true);

        self.issue_trbs(&[setup.into(), data.into(), status.into()])
            .await;

        b.max_packet_size()
    }

    pub(in crate::device::pci::xhci) async fn set_configure(&mut self, config_val: u8) {
        let setup = *transfer_trb::SetupStage::default()
            .set_transfer_type(TransferType::No)
            .set_trb_transfer_length(8)
            .set_interrupt_on_completion(false)
            .set_request_type(0)
            .set_request(9)
            .set_value(config_val.into())
            .set_length(0);

        let status = *transfer_trb::StatusStage::default().set_interrupt_on_completion(true);

        self.issue_trbs(&[setup.into(), status.into()]).await;
    }

    pub(in crate::device::pci::xhci) async fn set_idle(&mut self) {
        let setup = *transfer_trb::SetupStage::default()
            .set_transfer_type(TransferType::No)
            .set_trb_transfer_length(8)
            .set_interrupt_on_completion(false)
            .set_request_type(0x21)
            .set_request(0x0a)
            .set_value(0)
            .set_length(0);

        let status = *transfer_trb::StatusStage::default().set_interrupt_on_completion(true);

        self.issue_trbs(&[setup.into(), status.into()]).await;
    }

    pub async fn get_configuration_descriptor(&mut self) -> PageBox<[u8]> {
        let b = PageBox::new_slice(0, 4096);

        let (setup, data, status) = Self::trbs_for_getting_descriptors(
            &b,
            DescTyIdx::new(descriptor::Ty::Configuration, 0),
        );

        self.issue_trbs(&[setup, data, status]).await;
        debug!("Got TRBs");
        b
    }

    pub async fn issue_normal_trb<T: ?Sized>(&mut self, b: &PageBox<T>) {
        let t = *Normal::default()
            .set_data_buffer_pointer(b.phys_addr().as_u64())
            .set_trb_transfer_length(b.bytes().as_usize().try_into().unwrap())
            .set_interrupt_on_completion(true);
        debug!("Normal TRB: {:X?}", t);
        self.issue_trbs(&[t.into()]).await;
    }

    fn trbs_for_getting_descriptors<T: ?Sized>(
        b: &PageBox<T>,
        t: DescTyIdx,
    ) -> (
        transfer_trb::Allowed,
        transfer_trb::Allowed,
        transfer_trb::Allowed,
    ) {
        let setup = *transfer_trb::SetupStage::default()
            .set_request_type(0b1000_0000)
            .set_request(Request::GetDescriptor as u8)
            .set_value(t.bits())
            .set_length(b.bytes().as_usize().try_into().unwrap())
            .set_trb_transfer_length(8)
            .set_transfer_type(TransferType::In);

        let data = *transfer_trb::DataStage::default()
            .set_data_buffer_pointer(b.phys_addr().as_u64())
            .set_trb_transfer_length(b.bytes().as_usize().try_into().unwrap())
            .set_direction(Direction::In);

        let status = *transfer_trb::StatusStage::default().set_interrupt_on_completion(true);

        (setup.into(), data.into(), status.into())
    }

    async fn issue_trbs(&mut self, ts: &[transfer_trb::Allowed]) -> Vec<Option<event::Allowed>> {
        self.channel.send_and_receive(ts).await
    }
}

struct Channel {
    ring: transfer::Ring,
    doorbell_writer: DoorbellWriter,
    waker: Arc<Spinlock<AtomicWaker>>,
}
impl Channel {
    fn new(doorbell_writer: DoorbellWriter) -> Self {
        Self {
            ring: transfer::Ring::new(),
            doorbell_writer,
            waker: Arc::new(Spinlock::new(AtomicWaker::new())),
        }
    }

    fn ring_addr(&self) -> PhysAddr {
        self.ring.phys_addr()
    }

    async fn send_and_receive(
        &mut self,
        trbs: &[transfer_trb::Allowed],
    ) -> Vec<Option<event::Allowed>> {
        let addrs = self.ring.enqueue(trbs);
        self.register_with_receiver(trbs, &addrs);
        self.write_to_doorbell();
        self.get_trbs(trbs, &addrs).await
    }

    fn register_with_receiver(&mut self, ts: &[transfer_trb::Allowed], addrs: &[PhysAddr]) {
        for (t, addr) in ts.iter().zip(addrs) {
            self.register_trb(t, *addr);
        }
    }

    fn register_trb(&mut self, t: &transfer_trb::Allowed, a: PhysAddr) {
        if t.interrupt_on_completion() {
            receiver::add_entry(a, self.waker.clone()).expect("Sender is already registered.");
        }
    }

    fn write_to_doorbell(&mut self) {
        self.doorbell_writer.write();
    }

    async fn get_trbs(
        &mut self,
        ts: &[transfer_trb::Allowed],
        addrs: &[PhysAddr],
    ) -> Vec<Option<event::Allowed>> {
        let mut v = Vec::new();
        for (t, a) in ts.iter().zip(addrs) {
            v.push(self.get_single_trb(t, *a).await);
        }
        v
    }

    async fn get_single_trb(
        &mut self,
        t: &transfer_trb::Allowed,
        addr: PhysAddr,
    ) -> Option<event::Allowed> {
        if t.interrupt_on_completion() {
            Some(ReceiveFuture::new(addr, self.waker.clone()).await)
        } else {
            None
        }
    }
}

pub struct DoorbellWriter {
    slot_id: u8,
    val: u32,
}
impl DoorbellWriter {
    pub fn new(slot_id: u8, val: u32) -> Self {
        Self { slot_id, val }
    }

    pub fn write(&mut self) {
        registers::handle(|r| {
            r.doorbell.update_at(self.slot_id.into(), |d| {
                d.set_doorbell_target(self.val.try_into().unwrap())
            })
        });
    }
}

pub struct DescTyIdx {
    ty: descriptor::Ty,
    i: u8,
}
impl DescTyIdx {
    pub fn new(ty: descriptor::Ty, i: u8) -> Self {
        Self { ty, i }
    }
    pub fn bits(self) -> u16 {
        (self.ty as u16) << 8 | u16::from(self.i)
    }
}

enum Request {
    GetDescriptor = 6,
}
