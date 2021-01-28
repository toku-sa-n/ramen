// SPDX-License-Identifier: GPL-3.0-or-later

mod exchanger;
mod port;
mod structures;
mod xhc;

use core::convert::TryInto;

use super::config::bar;
use crate::{
    mem::accessor::Mappers,
    multitask::{self, task::Task},
};
use alloc::sync::Arc;
use conquer_once::spin::OnceCell;
use extended_capabilities::ExtendedCapability;
use spinning_top::Spinlock;
use structures::{
    dcbaa,
    ring::{command, event},
    scratchpad,
};
use x86_64::PhysAddr;
use xhci::{extended_capabilities, registers::Registers};

static REGISTERS: OnceCell<Spinlock<Registers<Mappers>>> = OnceCell::uninit();
static EXTENDED_CAPABILITIES: OnceCell<Spinlock<Option<extended_capabilities::List<Mappers>>>> =
    OnceCell::uninit();

pub async fn task() {
    if init_statics().is_err() {
        warn!("xHC not found.");
        return;
    }

    let event_ring = init();

    port::spawn_all_connected_port_tasks();

    multitask::add(Task::new_poll(event::task(event_ring)));

    info!("Issuing the NOOP trb.");
    exchanger::command::noop().await;
}

fn init_statics() -> Result<(), XhcNotFound> {
    match iter_devices().next() {
        Some(a) => {
            init_registers(a);
            init_extended_capabilities(a);
            Ok(())
        }
        None => Err(XhcNotFound),
    }
}

fn init_registers(mmio_base: PhysAddr) {
    REGISTERS
        .try_init_once(|| {
            Spinlock::new(
                // SAFETY: The address is the correct one and the Registers are accessed only through
                // this static.
                unsafe {
                    Registers::new(mmio_base.as_u64().try_into().unwrap(), Mappers::user())
                        .expect("The MMIO base address of the xHC is not aligned correctly.")
                },
            )
        })
        .expect("Failed to initialize `REGISTERS`.")
}

fn init_extended_capabilities(mmio_base: PhysAddr) {
    let hccparams1 = handle_registers(|r| r.capability.hccparams1.read());

    EXTENDED_CAPABILITIES
        .try_init_once(|| {
            Spinlock::new(
                // SAFETY: The address is the correct one and the Extended Capabilities are accessed only through
                // this static.
                unsafe {
                    extended_capabilities::List::new(
                        mmio_base.as_u64().try_into().unwrap(),
                        hccparams1,
                        Mappers::user(),
                    )
                },
            )
        })
        .expect("Failed to initialize `EXTENDED_CAPABILITIES`.");
}

#[derive(Debug)]
struct XhcNotFound;

/// Handle xHCI registers.
///
/// To avoid deadlocking, this method takes a closure. Caller is supposed not to call this method
/// inside the closure, otherwise a deadlock will happen.
///
/// Alternative implementation is to define a method which returns `impl Deref<Target =
/// Registers>`, but this will expand the scope of the mutex guard, increasing the possibility of
/// deadlocks.
fn handle_registers<T, U>(f: T) -> U
where
    T: Fn(&mut Registers<Mappers>) -> U,
{
    let mut r = REGISTERS.try_get().unwrap().lock();
    f(&mut r)
}

fn iter_extended_capabilities() -> Option<
    impl Iterator<Item = Result<ExtendedCapability<Mappers>, extended_capabilities::NotSupportedId>>,
> {
    Some(
        EXTENDED_CAPABILITIES
            .try_get()
            .expect("`EXTENDED_CAPABILITIES` is not initialized.`")
            .lock()
            .as_mut()?
            .into_iter(),
    )
}

fn init() -> event::Ring {
    let mut event_ring = event::Ring::new();
    let command_ring = Arc::new(Spinlock::new(command::Ring::new()));

    xhc::init();

    event_ring.init();
    command_ring.lock().init();
    dcbaa::init();
    scratchpad::init();
    exchanger::command::init(command_ring);

    xhc::run();
    xhc::ensure_no_error_occurs();

    event_ring
}

fn iter_devices() -> impl Iterator<Item = PhysAddr> {
    super::iter_devices().filter_map(|device| {
        if device.is_xhci() {
            Some(device.base_address(bar::Index::new(0)))
        } else {
            None
        }
    })
}
