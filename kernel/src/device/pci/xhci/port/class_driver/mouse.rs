// SPDX-License-Identifier: GPL-3.0-or-later

use crate::device::pci::xhci::port::init::fully_operational::FullyOperational;
use page_box::PageBox;
use xhci::context::EndpointType;

pub(in crate::device::pci::xhci::port) async fn task(eps: FullyOperational) {
    let mut m = Mouse::new(eps);
    loop {
        m.get_packet().await;
        m.print_buf();
    }
}

pub struct Mouse {
    ep: FullyOperational,
    buf: PageBox<[i8; 4]>,
}
impl Mouse {
    pub(in crate::device::pci::xhci::port) fn new(ep: FullyOperational) -> Self {
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
