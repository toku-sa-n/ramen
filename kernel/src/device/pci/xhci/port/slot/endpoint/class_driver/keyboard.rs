// SPDX-License-Identifier: GPL-3.0-or-later

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{
    device::pci::xhci::{port::slot::endpoint, structures::context::EndpointType},
    mem::allocator::page_box::PageBox,
};

pub async fn task(mut kbd: Keyboard) {
    loop {
        kbd.get_packet().await;
        kbd.print_buf();
        kbd.reset_buf();
    }
}

pub struct Keyboard {
    ep: endpoint::Collection,
    buf: PageBox<[u8; 8]>,
}
impl Keyboard {
    pub fn new(ep: endpoint::Collection) -> Self {
        Self {
            ep,
            buf: PageBox::new([0; 8]),
        }
    }

    async fn get_packet(&mut self) {
        self.issue_normal_trb().await;
        self.wait_until_packet_is_sent().await;
    }

    async fn issue_normal_trb(&mut self) {
        for e in &mut self.ep {
            if e.ty() == EndpointType::InterruptIn {
                e.sender.issue_normal_trb(&self.buf).await;
            }
        }
    }

    async fn wait_until_packet_is_sent(&self) {
        PacketWaiterFuture::new(self).await
    }

    fn print_buf(&self) {
        info!("Keyboard packet: {:?}", self.buf);
    }

    fn reset_buf(&mut self) {
        *self.buf = [0; 8];
    }
}

struct PacketWaiterFuture<'a> {
    kbd: &'a Keyboard,
}
impl<'a> PacketWaiterFuture<'a> {
    fn new(kbd: &'a Keyboard) -> Self {
        Self { kbd }
    }
}
impl<'a> Future for PacketWaiterFuture<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if *self.kbd.buf == [0; 8] {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
