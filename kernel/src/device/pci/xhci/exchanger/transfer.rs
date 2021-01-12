// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;

use super::receiver::{self, ReceiveFuture};
use crate::{
    device::pci::xhci::{
        self,
        structures::{
            descriptor,
            ring::{
                event::trb::completion::Completion,
                transfer::{
                    self,
                    trb::control::{Control, Direction, Request},
                },
            },
        },
    },
    mem::allocator::page_box::PageBox,
};
use alloc::{sync::Arc, vec::Vec};
use futures_util::task::AtomicWaker;
use spinning_top::Spinlock;
use transfer::trb::{
    control::{DataStage, DescTyIdx, SetupStage, StatusStage},
    Normal, Trb,
};
use x86_64::PhysAddr;

pub struct Sender {
    ring: transfer::Ring,
    doorbell_writer: DoorbellWriter,
    waker: Arc<Spinlock<AtomicWaker>>,
}
impl Sender {
    pub fn new(doorbell_writer: DoorbellWriter) -> Self {
        Self {
            ring: transfer::Ring::new(),
            doorbell_writer,
            waker: Arc::new(Spinlock::new(AtomicWaker::new())),
        }
    }

    pub fn ring_addr(&self) -> PhysAddr {
        self.ring.phys_addr()
    }

    pub async fn get_device_descriptor(&mut self) -> PageBox<descriptor::Device> {
        let b = PageBox::user(descriptor::Device::default());

        let (setup, data, status) =
            Self::trbs_for_getting_descriptors(&b, DescTyIdx::new(descriptor::Ty::Device, 0));

        self.issue_trbs(&[setup, data, status]).await;
        b
    }

    pub async fn get_configuration_descriptor(&mut self) -> PageBox<[u8]> {
        let b = PageBox::user_slice(0, 4096);

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
            .set_buf_ptr(b.phys_addr())
            .set_transfer_length(b.bytes())
            .set_ioc(true);
        let t = Trb::Normal(t);
        debug!("Normal TRB: {:X?}", t);
        self.issue_trbs(&[t]).await;
    }

    fn trbs_for_getting_descriptors<T: ?Sized>(b: &PageBox<T>, t: DescTyIdx) -> (Trb, Trb, Trb) {
        let setup = *SetupStage::default()
            .set_request_type(0b1000_0000)
            .set_request(Request::GetDescriptor)
            .set_value(t.bits())
            .set_length(b.bytes().as_usize().try_into().unwrap())
            .set_trb_transfer_length(8)
            .set_trt(3);
        let setup = Trb::Control(Control::Setup(setup));

        let data = *DataStage::default()
            .set_data_buf(b.phys_addr())
            .set_transfer_length(b.bytes().as_usize().try_into().unwrap())
            .set_dir(Direction::In);
        let data = Trb::Control(Control::Data(data));

        let status = *StatusStage::default().set_ioc(true);
        let status = Trb::Control(Control::Status(status));

        (setup, data, status)
    }

    async fn issue_trbs(&mut self, ts: &[Trb]) -> Vec<Option<Completion>> {
        let addrs = self.ring.enqueue(ts);
        self.register_with_receiver(ts, &addrs);
        self.write_to_doorbell();
        self.get_trb(ts, &addrs).await
    }

    fn register_with_receiver(&mut self, ts: &[Trb], addrs: &[PhysAddr]) {
        for (t, addr) in ts.iter().zip(addrs) {
            self.register_trb(t, *addr);
        }
    }

    fn register_trb(&mut self, t: &Trb, a: PhysAddr) {
        if t.ioc() {
            receiver::add_entry(a, self.waker.clone()).expect("Sender is already registered.");
        }
    }

    fn write_to_doorbell(&mut self) {
        self.doorbell_writer.write();
    }

    async fn get_trb(&mut self, ts: &[Trb], addrs: &[PhysAddr]) -> Vec<Option<Completion>> {
        let mut v = Vec::new();
        for (t, a) in ts.iter().zip(addrs) {
            v.push(self.get_single_trb(t, *a).await);
        }
        v
    }

    async fn get_single_trb(&mut self, t: &Trb, addr: PhysAddr) -> Option<Completion> {
        if t.ioc() {
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
        xhci::handle_registers(|r| {
            let d = &mut r.doorbell_array;
            d.update(self.slot_id.into(), |d| *d = self.val);
        });
    }
}
