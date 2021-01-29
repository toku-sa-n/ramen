// SPDX-License-Identifier: GPL-3.0-or-later

use super::registers;
use crate::mem::accessor::Mappers;
use conquer_once::spin::OnceCell;
use core::convert::TryInto;
use spinning_top::Spinlock;
use x86_64::PhysAddr;
use xhci::{extended_capabilities, ExtendedCapability};

static EXTENDED_CAPABILITIES: OnceCell<Spinlock<Option<extended_capabilities::List<Mappers>>>> =
    OnceCell::uninit();

pub(in crate::device::pci::xhci) fn init(mmio_base: PhysAddr) {
    let hccparams1 = registers::handle(|r| r.capability.hccparams1.read());

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

pub(in crate::device::pci::xhci) fn iter() -> Option<
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
