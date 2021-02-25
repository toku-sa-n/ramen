// SPDX-License-Identifier: GPL-3.0-or-later

use conquer_once::spin::OnceCell;
use core::convert::TryInto;
use memory::accessor::Mappers;
use spinning_top::Spinlock;
use x86_64::PhysAddr;
use xhci::Registers;

static REGISTERS: OnceCell<Spinlock<Registers<Mappers>>> = OnceCell::uninit();

pub(in crate::device::pci::xhci) fn init(mmio_base: PhysAddr) {
    REGISTERS
        .try_init_once(|| {
            Spinlock::new(
                // SAFETY: The address is the correct one and the Registers are accessed only through
                // this static.
                unsafe { Registers::new(mmio_base.as_u64().try_into().unwrap(), Mappers::new()) },
            )
        })
        .expect("Failed to initialize `REGISTERS`.")
}

/// Handle xHCI registers.
///
/// To avoid deadlocking, this method takes a closure. Caller is supposed not to call this method
/// inside the closure, otherwise a deadlock will happen.
///
/// Alternative implementation is to define a method which returns `impl Deref<Target =
/// Registers>`, but this will expand the scope of the mutex guard, increasing the possibility of
/// deadlocks.
pub(in crate::device::pci::xhci) fn handle<T, U>(f: T) -> U
where
    T: FnOnce(&mut Registers<Mappers>) -> U,
{
    let mut r = REGISTERS.try_get().unwrap().lock();
    f(&mut r)
}
