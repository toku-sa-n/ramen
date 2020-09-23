// SPDX-License-Identifier: GPL-3.0-or-later

use bitfield::bitfield;

bitfield! {
    pub struct MsiX([u8]);
    u32;
    capability_id, _: 7, 0;
    table_size, _: 25, 16;
}
