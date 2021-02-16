// SPDX-License-Identifier: GPL-3.0-or-later

use crate::device::pci::xhci::port::endpoint;
use page_box::PageBox;
use xhci::context::EndpointType;

pub async fn task(eps: endpoint::AddressAssigned) {
    let mut m = Mouse::new(eps);
    loop {
        m.get_packet().await;
        m.print_buf();
    }
}

pub struct Mouse {
    ep: endpoint::AddressAssigned,
    buf: PageBox<[i8; 4]>,
}
impl Mouse {
    pub fn new(ep: endpoint::AddressAssigned) -> Self {
        Self {
            ep,
            buf: [0; 4].into(),
        }
    }

    async fn get_packet(&mut self) {
        self.issue_normal_trb().await;
    }

    async fn issue_normal_trb(&mut self) {
        self.ep
            .issue_normal_trb(&self.buf, EndpointType::InterruptIn)
            .await
            .expect("Failed to send a Normal TRB.");
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
