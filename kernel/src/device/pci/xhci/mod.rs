// SPDX-License-Identifier: GPL-3.0-or-later

mod context;
mod dcbaa;
mod exchanger;
mod port;
mod registers;
mod ring;
mod xhc;

use super::config::bar;
use crate::multitask::task::{self, Task};
use alloc::rc::Rc;
use core::cell::RefCell;
use dcbaa::DeviceContextBaseAddressArray;
use exchanger::command::{Receiver, Sender};
use futures_intrusive::sync::LocalMutex;
use registers::Registers;
use ring::{command, event};
use xhc::Xhc;

pub async fn task(task_collection: Rc<RefCell<task::Collection>>) {
    let registers = Rc::new(RefCell::new(iter_devices().next().unwrap()));
    let (event_ring, dcbaa, runner, command_completion_receiver) =
        init(&registers, &task_collection);

    port::spawn_tasks(&runner, &dcbaa, &registers, &task_collection);

    task_collection
        .borrow_mut()
        .add_task_as_woken(Task::new(event::task(
            event_ring,
            command_completion_receiver,
        )));
}

// FIXME
#[allow(clippy::type_complexity)]
fn init(
    registers: &Rc<RefCell<Registers>>,
    task_collection: &Rc<RefCell<task::Collection>>,
) -> (
    event::Ring,
    Rc<RefCell<DeviceContextBaseAddressArray>>,
    Rc<LocalMutex<Sender>>,
    Rc<RefCell<Receiver>>,
) {
    let mut xhc = Xhc::new(registers.clone());
    let mut event_ring = event::Ring::new(registers.clone(), task_collection.clone());
    let command_ring = Rc::new(RefCell::new(command::Ring::new(registers.clone())));
    let dcbaa = Rc::new(RefCell::new(DeviceContextBaseAddressArray::new(
        registers.clone(),
    )));
    let command_completion_receiver = Rc::new(RefCell::new(Receiver::new()));
    let command_runner = Rc::new(LocalMutex::new(
        Sender::new(command_ring.clone(), command_completion_receiver.clone()),
        false,
    ));

    xhc.init();

    event_ring.init();
    command_ring.borrow_mut().init();
    dcbaa.borrow_mut().init();

    xhc.run();

    (
        event_ring,
        dcbaa,
        command_runner,
        command_completion_receiver,
    )
}

pub fn iter_devices() -> impl Iterator<Item = Registers> {
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
