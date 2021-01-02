// SPDX-License-Identifier: GPL-3.0-or-later

use bitfield::bitfield;

bitfield! {
    #[repr(transparent)]
    pub struct UsbLegacySupport(u32);

    id, _: 7, 0;
    pub bios_owns_hc, _: 16;
    pub os_owns_hc, os_request_ownership: 24;
}
