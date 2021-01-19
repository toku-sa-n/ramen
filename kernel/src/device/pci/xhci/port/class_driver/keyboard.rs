// SPDX-License-Identifier: GPL-3.0-or-later

use crate::device::pci::xhci::{port::endpoint, structures::context::EndpointType};
use alloc::string::String;
use page_box::PageBox;
use spinning_top::Spinlock;

const LOWER_ALPHABETS: &str = "abcdefghijklmnopqrstuvwxyz";

static STR: Spinlock<String> = Spinlock::new(String::new());

pub async fn task(eps: endpoint::Collection) {
    let mut k = Keyboard::new(eps);
    loop {
        k.get_packet().await;
        k.store_key();
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
                e.issue_normal_trb(&self.buf).await;
            }
        }
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
