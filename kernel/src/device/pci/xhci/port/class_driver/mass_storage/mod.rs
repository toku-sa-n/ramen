// SPDX-License-Identifier: GPL-3.0-or-later

use crate::device::pci::xhci::port::endpoint;

pub async fn task(eps: endpoint::Collection) {
    let m = MassStorage::new(eps);
    info!("This is the task of USB Mass Storage.");
}

struct MassStorage {
    eps: endpoint::Collection,
}
impl MassStorage {
    fn new(eps: endpoint::Collection) -> Self {
        Self { eps }
    }
}

#[repr(C, packed)]
struct CommandBlockWrapper {
    header: CommandBlockWrapperHeader,
    data: CommandDataBlock,
}
impl CommandBlockWrapper {
    fn new(header: CommandBlockWrapperHeader, data: CommandDataBlock) -> Self {
        Self { header, data }
    }
}

#[repr(C, packed)]
#[derive(Builder)]
#[builder(no_std)]
struct CommandBlockWrapperHeader {
    #[builder(default = "CommandBlockWrapperHeader::SIGNATURE")]
    signature: u32,
    #[builder(default = "0")]
    tag: u32,
    transfer_length: u32,
    flags: u8,
    lun: u8,
    command_len: u8,
}
impl CommandBlockWrapperHeader {
    const SIGNATURE: u32 = 0x43425355;
}

#[repr(transparent)]
#[derive(Default)]
struct CommandDataBlock([u8; 16]);
impl CommandDataBlock {
    fn inquiry() -> Self {
        let mut b = Self::default();
        b.0[0] = 0x12;
        b.0[4] = 0x24;
        b
    }
}
