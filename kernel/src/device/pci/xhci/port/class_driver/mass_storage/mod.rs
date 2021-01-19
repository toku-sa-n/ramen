// SPDX-License-Identifier: GPL-3.0-or-later

mod scsi;

use crate::device::pci::xhci::{port::endpoint, structures::context::EndpointType};
use page_box::PageBox;
use scsi::{CommandBlockWrapper, CommandBlockWrapperHeaderBuilder, CommandDataBlock};

pub async fn task(eps: endpoint::Collection) {
    let mut m = MassStorage::new(eps);
    info!("This is the task of USB Mass Storage.");
    let b = m.inquiry().await;
    info!("Buffer: {:?}", b);
}

struct MassStorage {
    eps: endpoint::Collection,
}
impl MassStorage {
    fn new(eps: endpoint::Collection) -> Self {
        Self { eps }
    }

    async fn inquiry(&mut self) -> PageBox<[u8; 36]> {
        let b = PageBox::new([0_u8; 36]);
        let header = CommandBlockWrapperHeaderBuilder::default()
            .transfer_length(36)
            .flags(0x80)
            .lun(0)
            .command_len(6)
            .build()
            .expect("Failed to build an inquiry command block wrapper.");
        let data = CommandDataBlock::inquiry();
        let wrapper = PageBox::new(CommandBlockWrapper::new(header, data));

        for ep in &mut self.eps {
            if ep.ty() == EndpointType::BulkOut {
                ep.issue_normal_trb(&wrapper).await;
            }
        }

        for ep in &mut self.eps {
            if ep.ty() == EndpointType::BulkIn {
                ep.issue_normal_trb(&b).await;
            }
        }

        b
    }
}
