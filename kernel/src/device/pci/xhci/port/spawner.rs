// SPDX-License-Identifier: GPL-3.0-or-later

use alloc::sync::Arc;
use multitask::task::Task;
use spinning_top::Spinlock;

use crate::{
    device::pci::xhci::exchanger::{command, receiver::Receiver},
    multitask, Futurelock,
};

use super::Port;

pub struct Spawner {
    sender: Arc<Futurelock<command::Sender>>,
    receiver: Arc<Spinlock<Receiver>>,
}
impl Spawner {
    pub fn new(
        sender: Arc<Futurelock<command::Sender>>,
        receiver: Arc<Spinlock<Receiver>>,
    ) -> Self {
        Self { sender, receiver }
    }

    pub fn scan_all_ports_and_spawn(&self) {
        let n = super::max_num();
        for i in 0..n {
            let _ = self.try_spawn(i + 1);
        }
    }

    fn try_spawn(&self, port_idx: u8) -> Result<(), PortNotConnected> {
        let p = Port::new(port_idx);
        if p.connected() {
            self.spawn(p);
            Ok(())
        } else {
            Err(PortNotConnected)
        }
    }

    fn spawn(&self, p: Port) {
        multitask::add(Task::new(super::task(
            p,
            self.sender.clone(),
            self.receiver.clone(),
        )));
    }
}

#[derive(Debug)]
struct PortNotConnected;
