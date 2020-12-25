// SPDX-License-Identifier: GPL-3.0-or-later

use alloc::sync::Arc;
use multitask::task::Task;
use spinning_top::Spinlock;

use crate::{
    device::pci::xhci::exchanger::{command, receiver::Receiver},
    multitask, Futurelock,
};

use super::Port;

struct Spawner {
    sender: Arc<Futurelock<command::Sender>>,
    receiver: Arc<Spinlock<Receiver>>,
}
impl Spawner {
    fn new(sender: Arc<Futurelock<command::Sender>>, receiver: Arc<Spinlock<Receiver>>) -> Self {
        Self { sender, receiver }
    }

    fn spawn(&self, port_idx: u8) {
        let p = Port::new(port_idx);
        multitask::add(Task::new(super::task(
            p,
            self.sender.clone(),
            self.receiver.clone(),
        )));
    }
}
