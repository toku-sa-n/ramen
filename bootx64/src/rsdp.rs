// SPDX-License-Identifier: GPL-3.0-or-later

use {
    uefi::{
        table::{Boot, SystemTable},
        Guid,
    },
    x86_64::PhysAddr,
};

const GUID_RSDP: Guid = Guid::from_values(0x8868_e871, 0xe4f1, 0x11d3, 0xbc22, 0x0080_c73c_8881);

/// # Panics
///
/// This function panics if the architecture does not have RSDP.
#[must_use]
pub fn get(st: &SystemTable<Boot>) -> PhysAddr {
    for c in st.config_table() {
        if c.guid == GUID_RSDP {
            return PhysAddr::new(c.address as u64);
        }
    }

    unimplemented!("Not implemented for architectures which do not have RSDP.");
}
