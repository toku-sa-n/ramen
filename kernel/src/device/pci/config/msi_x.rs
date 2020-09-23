// SPDX-License-Identifier: GPL-3.0-or-later

use bitfield::bitfield;

bitfield! {
    pub struct MsiX([u8]);
    u32;
    capability_id, _: 7, 0;
    table_size, _: 25, 16;
}

bitfield! {
    struct MessageAddress(u64);
    u32;
    destination_id, set_destination_id: 19, 12;
    redirection_hint, set_redirection_hint: 3;
    destination_mode, _: 2;
}

bitfield! {
    struct MessageData(u32);

    trigger_mode, set_trigger_mode: 15;
    level, set_level: 14;
    delivery_mode, set_delivery_mode: 10, 8;
    vector, set_vector: 7, 0;
}

struct VectorControl(u32);
