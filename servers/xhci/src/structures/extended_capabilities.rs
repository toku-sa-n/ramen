// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::registers,
    conquer_once::spin::OnceCell,
    core::convert::TryInto,
    ralib::mem::accessor::Mapper,
    spinning_top::Spinlock,
    x86_64::PhysAddr,
    xhci::{extended_capabilities, ExtendedCapability},
};

static EXTENDED_CAPABILITIES: OnceCell<Spinlock<Option<extended_capabilities::List<Mapper>>>> =
    OnceCell::uninit();

pub(crate) unsafe fn init(mmio_base: PhysAddr) {
    let hccparams1 = registers::handle(|r| r.capability.hccparams1.read_volatile());

    EXTENDED_CAPABILITIES
        .try_init_once(|| unsafe {
            Spinlock::new(extended_capabilities::List::new(
                mmio_base.as_u64().try_into().unwrap(),
                hccparams1,
                Mapper,
            ))
        })
        .expect("Failed to initialize `EXTENDED_CAPABILITIES`.");
}

pub(crate) fn iter() -> Option<
    impl Iterator<Item = Result<ExtendedCapability<Mapper>, extended_capabilities::NotSupportedId>>,
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
