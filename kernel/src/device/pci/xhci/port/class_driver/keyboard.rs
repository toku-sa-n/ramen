// SPDX-License-Identifier: GPL-3.0-or-later

use alloc::string::String;
use spinning_top::Spinlock;

use crate::{
    device::pci::xhci::{port::endpoint, structures::context::EndpointType},
    mem::allocator::page_box::PageBox,
};

const LOWER_ALPHABETS: &str = "abcdefghijklmnopqrstuvwxyz";

static STR: Spinlock<String> = Spinlock::new(String::new());

pub async fn task(mut kbd: Keyboard) {
    loop {
        kbd.get_packet().await;
        kbd.store_key();
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
            buf: PageBox::user([0; 8]),
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
