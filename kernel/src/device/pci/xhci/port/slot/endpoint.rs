// SPDX-License-Identifier: GPL-3.0-or-later

use crate::device::pci::xhci::structures::{context::Context, descriptor};
use alloc::rc::Rc;
use core::cell::RefCell;

pub struct Endpoint {
    desc: descriptor::Endpoint,
    cx: Rc<RefCell<Context>>,
}
impl Endpoint {
    pub fn new(desc: descriptor::Endpoint, cx: Rc<RefCell<Context>>) -> Self {
        Self { desc, cx }
    }
}
