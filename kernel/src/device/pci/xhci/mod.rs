// SPDX-License-Identifier: GPL-3.0-or-later

mod exchanger;
mod port;
mod structures;
mod xhc;

use super::config::bar;
use crate::{
    multitask::task::{self, Task},
    Futurelock,
};
use alloc::sync::Arc;
use exchanger::{command::Sender, receiver::Receiver};
use spinning_top::Spinlock;
use structures::{
    dcbaa::DeviceContextBaseAddressArray,
    registers::Registers,
    ring::{command, event},
};
use xhc::Xhc;

pub async fn task() {
    let registers = Arc::new(Spinlock::new(iter_devices().next().unwrap()));
    let (event_ring, dcbaa, runner, command_completion_receiver) = init(&registers);

    port::spawn_tasks(&runner, &dcbaa, &registers, &command_completion_receiver);

    task::COLLECTION
        .lock()
        .add_task_as_woken(Task::new_poll(event::task(
            event_ring,
            command_completion_receiver,
        )));
}

// FIXME
#[allow(clippy::type_complexity)]
fn init(
    registers: &Arc<Spinlock<Registers>>,
) -> (
    event::Ring,
    Arc<Spinlock<DeviceContextBaseAddressArray>>,
    Arc<Futurelock<Sender>>,
    Arc<Spinlock<Receiver>>,
) {
    let mut xhc = Xhc::new(registers.clone());
    let mut event_ring = event::Ring::new(registers.clone());
    let command_ring = Arc::new(Spinlock::new(command::Ring::new(registers.clone())));
    let dcbaa = Arc::new(Spinlock::new(DeviceContextBaseAddressArray::new(
        registers.clone(),
    )));
    let receiver = Arc::new(Spinlock::new(Receiver::new()));
    let sender = Arc::new(Futurelock::new(
        Sender::new(command_ring.clone(), receiver.clone()),
        false,
    ));

    xhc.init();

    event_ring.init();
    command_ring.lock().init();
    dcbaa.lock().init();

    xhc.run();

    (event_ring, dcbaa, sender, receiver)
}

fn iter_devices() -> impl Iterator<Item = Registers> {
    super::iter_devices().filter_map(|device| {
        if device.is_xhci() {
            // Safety: This operation is safe because MMIO base address is generated from the 0th
            // BAR.
            Some(unsafe { Registers::new(device.base_address(bar::Index::new(0))) })
        } else {
            None
        }
    })
}
