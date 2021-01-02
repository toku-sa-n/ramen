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
    buf: PageBox<[i8; 4]>,
}
impl Mouse {
    pub fn new(ep: endpoint::Collection) -> Self {
        Self {
            ep,
            buf: PageBox::user([0; 4]),
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
        info!(
            "Button: {} {} {}, X: {}, Y: {}",
            self.buf[0] & 1 == 1,
            self.buf[0] & 2 == 2,
            self.buf[0] & 4 == 4,
            self.buf[1],
            self.buf[2]
        );
    }
}
