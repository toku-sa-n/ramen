// SPDX-License-Identifier: GPL-3.0-or-later

use alloc::sync::Arc;
use spinning_top::Spinlock;

use crate::{
    device::pci::xhci::exchanger::{command, receiver::Receiver},
    Futurelock,
};

struct Spawner {
    sender: Arc<Futurelock<command::Sender>>,
    receiver: Arc<Spinlock<Receiver>>,
}
impl Spawner {
    fn new(sender: Arc<Futurelock<command::Sender>>, receiver: Arc<Spinlock<Receiver>>) -> Self {
        Self { sender, receiver }
    }
}
