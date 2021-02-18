// SPDX-License-Identifier: GPL-3.0-or-later

use crate::device::pci::xhci::port::init::fully_operational::FullyOperational;
use alloc::string::String;
use page_box::PageBox;
use spinning_top::Spinlock;
use xhci::context::EndpointType;

const LOWER_ALPHABETS: &str = "abcdefghijklmnopqrstuvwxyz";

static STR: Spinlock<String> = Spinlock::new(String::new());

pub(in crate::device::pci::xhci::port) async fn task(eps: FullyOperational) {
    let mut k = Keyboard::new(eps);
    loop {
        k.get_packet().await;
        k.store_key();
    }
}

pub struct Keyboard {
    ep: FullyOperational,
    buf: PageBox<[u8; 8]>,
}
impl Keyboard {
    pub(in crate::device::pci::xhci::port) fn new(ep: FullyOperational) -> Self {
        Self {
            ep,
            buf: [0; 8].into(),
        }
    }

    async fn get_packet(&mut self) {
        self.issue_normal_trb().await;
    }

    async fn issue_normal_trb(&mut self) {
        self.ep
            .issue_normal_trb(&self.buf, EndpointType::InterruptIn)
            .await
            .expect("Failed to send a Normal TRB");
    }

    fn store_key(&self) {
        for c in self.buf.iter().skip(2) {
            if *c >= 4 && *c <= 0x1d {
                STR.lock()
                    .push(LOWER_ALPHABETS.chars().nth((c - 4).into()).unwrap());
            } else if *c == 0x28 {
                info!("{}", STR.lock());
                *STR.lock() = String::new();
            }
        }
    }
}
