// SPDX-License-Identifier: GPL-3.0-or-later

use os_units::Bytes;
use x86_64::PhysAddr;

use crate::mem::accessor::Accessor;

const REGISTER_BASE: PhysAddr = PhysAddr::new_truncate(0xfee0_0000);

pub fn end_of_interrupt() {
    // SAFETY: This operation is safe because `REGISTER_BASE` is the valid address to the Local APIC
    // registers.
    let mut r = unsafe { Accessor::<u32>::new(REGISTER_BASE, Bytes::new(0xb0)) };
    r.write(0);
}
