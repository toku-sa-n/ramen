// SPDX-License-Identifier: GPL-3.0-or-later

use x86_64::PhysAddr;

const REGISTER_BASE: PhysAddr = PhysAddr::new_truncate(0xfee0_0000);

pub(crate) fn end_of_interrupt() {
    // SAFETY: This operation is safe because `REGISTER_BASE` is the valid address to the Local APIC
    // registers.
    let mut r = unsafe { crate::mem::accessor::new::<u32>(REGISTER_BASE + 0xb0_usize) };
    r.write(0);
}
