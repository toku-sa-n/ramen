// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;

use acpi::AcpiTables;
use x86_64::PhysAddr;

use crate::mem::allocator;

/// SAFETY: This method is unsafe because the caller must ensure that `rsdb` is a valid RSDB.
/// Otherwise this function will break memory safety by dereferencing to an invalid address.
pub unsafe fn get(rsdb: PhysAddr) -> AcpiTables<allocator::acpi::Mapper> {
    let mapper = allocator::acpi::Mapper;
    AcpiTables::from_rsdp(mapper, rsdb.as_u64().try_into().unwrap()).unwrap()
}
