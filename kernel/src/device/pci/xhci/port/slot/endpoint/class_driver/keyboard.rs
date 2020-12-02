// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    device::pci::xhci::{port::slot::endpoint, structures::context::EndpointType},
    mem::allocator::page_box::PageBox,
};

pub async fn task(mut kbd: Keyboard) {
    loop {
        kbd.get_packet().await;
        kbd.print_buf();
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
    }

    async fn issue_normal_trb(&mut self) {
        for e in &mut self.ep {
            if e.ty() == EndpointType::InterruptIn {
                e.sender.issue_normal_trb(&self.buf).await;
            }
        }
    }

    fn print_buf(&self) {
        debug!("Keyboard packet: {:?}", self.buf);
    }
}
