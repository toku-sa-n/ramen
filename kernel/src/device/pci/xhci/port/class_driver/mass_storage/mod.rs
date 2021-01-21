// SPDX-License-Identifier: GPL-3.0-or-later

mod scsi;

use crate::device::pci::xhci::{port::endpoint, structures::context::EndpointType};
use core::convert::TryFrom;
use page_box::PageBox;
use scsi::{
    CommandBlockWrapper, CommandBlockWrapperHeaderBuilder, CommandDataBlock, CommandStatus,
    Inquiry, RawInquiry,
};

pub async fn task(eps: endpoint::Collection) {
    let mut m = MassStorage::new(eps);
    info!("This is the task of USB Mass Storage.");
    let b = m.inquiry().await;
    info!("Inquiry Command: {:?}", b);
}

struct MassStorage {
    eps: endpoint::Collection,
}
impl MassStorage {
    fn new(eps: endpoint::Collection) -> Self {
        Self { eps }
    }

    async fn inquiry(&mut self) -> Result<Inquiry, scsi::Invalid> {
        let mut b = PageBox::from(CommandStatus::<RawInquiry>::default());
        let header = CommandBlockWrapperHeaderBuilder::default()
            .transfer_length(36)
            .flags(0x80)
            .lun(0)
            .command_len(6)
            .build()
            .expect("Failed to build an inquiry command block wrapper.");
        let data = CommandDataBlock::inquiry();
        let mut wrapper = PageBox::from(CommandBlockWrapper::new(header, data));

        self.send_scsi_command(&mut wrapper, &mut b).await;

        info!("Status: {:?}", b.wrapper().status());
        let b: CommandStatus<Inquiry> = b.try_into()?.clone();
        Ok(b.status())
    }

    async fn send_scsi_command<T: Copy>(
        &mut self,
        c: &mut PageBox<CommandBlockWrapper>,
        recv: &mut PageBox<CommandStatus<T>>,
    ) {
        self.send_command_block_wrapper(c).await;
        self.receive_command_status(recv).await;
    }

    async fn send_command_block_wrapper(&mut self, c: &mut PageBox<CommandBlockWrapper>) {
        self.eps
            .issue_normal_trb(c, EndpointType::BulkOut)
            .await
            .expect("Failed to send a SCSI command.");
    }

    async fn receive_command_status<T: Copy>(&mut self, c: &mut PageBox<CommandStatus<T>>) {
        self.eps
            .issue_normal_trb(c, EndpointType::BulkIn)
            .await
            .expect("Failed to receive a SCSI status");
    }
}
