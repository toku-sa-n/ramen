// SPDX-License-Identifier: GPL-3.0-or-later

use crate::device::pci::xhci::{port::endpoint, structures::context::EndpointType};
use page_box::PageBox;

pub async fn task(eps: endpoint::Collection) {
    let mut m = Mouse::new(eps);
    loop {
        m.get_packet().await;
        m.print_buf();
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
            buf: PageBox::new([0; 4]),
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
