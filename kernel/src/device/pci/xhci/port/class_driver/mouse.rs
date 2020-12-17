// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    device::pci::xhci::{port::endpoint, structures::context::EndpointType},
    mem::allocator::page_box::PageBox,
};

pub async fn task(mut mouse: Mouse) {
    loop {
        mouse.get_packet().await;
        mouse.print_buf();
    }
}

pub struct Mouse {
    ep: endpoint::Collection,
    buf: PageBox<[u8; 4]>,
}
impl Mouse {
    pub fn new(ep: endpoint::Collection) -> Self {
        Self {
            ep,
            buf: PageBox::new([0; 4]),
        }
    }

    async fn get_packet(&mut self) {
        self.issue_normal_trb().await;
    }

    async fn issue_normal_trb(&mut self) {
        for e in &mut self.ep {
            if e.ty() == EndpointType::InterruptIn {
                e.issue_normal_trb(&self.buf).await;
            }
        }
    }

    fn print_buf(&self) {
        info!("{:?}", self.buf);
    }
}
