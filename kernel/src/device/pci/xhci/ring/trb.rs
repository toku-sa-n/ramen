// SPDX-License-Identifier: GPL-3.0-or-later

use bitfield::bitfield;

bitfield! {
    #[repr(transparent)]
    pub struct Noop(u128);

    _, set_cycle_bit: 96;
    _, set_trb_type: 96+15, 96+10;
}
impl Noop {
    fn new(cycle_bit: CycleBit) -> Self {
        let mut noop = Noop(0);
        noop.set_cycle_bit(cycle_bit.into());
        noop.set_trb_type(8);

        noop
    }
}

struct CycleBit(bool);
impl From<CycleBit> for bool {
    fn from(cycle_bit: CycleBit) -> Self {
        cycle_bit.0
    }
}
