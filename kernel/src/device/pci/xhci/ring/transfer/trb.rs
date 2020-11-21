// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::{raw, CycleBit};
use bitfield::bitfield;
use core::convert::TryFrom;
use os_units::Bytes;
use x86_64::PhysAddr;

pub enum Trb {}
impl Trb {
    pub const SIZE: Bytes = Bytes::new(16);

    pub fn new_link(a: PhysAddr, c: CycleBit) -> Self {
        unimplemented!()
    }
}
impl TryFrom<raw::Trb> for Trb {
    type Error = Error;

    fn try_from(r: raw::Trb) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}
impl From<Trb> for raw::Trb {
    fn from(t: Trb) -> Self {
        unimplemented!()
    }
}

pub type SetupStage = SetupStageStructure<[u32; 4]>;
bitfield! {
    #[repr(transparent)]
    pub struct SetupStageStructure([u32]);
    impl Debug;
    u128, _, set_bm_request_type: 7, 0;
    u128, _, set_b_request: 15, 8;
    u128, _, set_w_value: 31, 16;
    u128, _, set_w_index: 32+15, 32;
    u128, _, set_w_length: 32+31, 32+16;
    u128, _, set_trb_transfer_length: 64+16, 64;
    u128, _, set_cycle_bit: 96;
    u128, _, set_trb_type: 96+15, 96+10;
    u128, _, set_trt: 96+17, 96+16;
}

pub type DataStage = DataStageStructure<[u32; 4]>;
bitfield! {
    #[repr(transparent)]
    pub struct DataStageStructure([u32]);
    impl Debug;
    u64, _, set_data_buffer_as_u64: 63, 0;
    u128, _, set_trb_transfer_length: 64+16, 64;
    u128, _, set_td_size: 64+21, 64+17;
    _, set_cycle_bit: 96;
    _, set_ioc: 96+5;
    u128, _, set_trb_type: 96+15, 96+10;
    _, set_dir: 96+16;
}

pub type StatusStage = StatusStageStructure<[u32; 4]>;
bitfield! {
    #[repr(transparent)]
    pub struct StatusStageStructure([u32]);
    impl Debug;
    _, set_cycle_bit: 96;
    u128, _, set_trb_type: 96+15, 96+10;
}

pub type TransferEvent = TransferEventStructure<[u32; 4]>;
bitfield! {
    #[repr(transparent)]
    pub struct TransferEventStructure([u32]);
    impl Debug;
}

#[derive(Debug)]
pub enum Error {
    UnrecognizedId,
}
