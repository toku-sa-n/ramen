// SPDX-License-Identifier: GPL-3.0-or-later

mod scsi;

use crate::device::pci::xhci::port::endpoint;
use page_box::PageBox;
use scsi::{
    response::{Inquiry, ReadFormatCapacities},
    CommandBlockWrapper, CommandBlockWrapperHeaderBuilder, CommandDataBlock, CommandStatusWrapper,
};
use xhci::context::EndpointType;

pub async fn task(eps: endpoint::Collection) {
    let mut m = MassStorage::new(eps);
    info!("This is the task of USB Mass Storage.");
    let b = m.inquiry().await;
    info!("Inquiry Command: {:?}", b);
    let b = m
        .read_format_capacities()
        .await
        .expect("Failed to send a read format capacities command.");
    info!("Len: {}", b.header.list_len);
    info!("{:?}", b);
}

struct MassStorage {
    eps: endpoint::Collection,
}
impl MassStorage {
    fn new(eps: endpoint::Collection) -> Self {
        Self { eps }
    }

    async fn inquiry(&mut self) -> Result<Inquiry, scsi::Invalid> {
        let header = CommandBlockWrapperHeaderBuilder::default()
            .transfer_length(36)
            .flags(0x80)
            .lun(0)
            .command_len(6)
            .build()
            .expect("Failed to build an inquiry command block wrapper.");
        let data = CommandDataBlock::inquiry();
        let mut wrapper = PageBox::from(CommandBlockWrapper::new(header, data));

        let (response, status): (PageBox<Inquiry>, _) = self.send_scsi_command(&mut wrapper).await;

        status.check_corruption();
        Ok(*response)
    }

    async fn read_format_capacities(&mut self) -> Result<ReadFormatCapacities, scsi::Invalid> {
        let header = CommandBlockWrapperHeaderBuilder::default()
            .transfer_length(0xfc)
            .flags(0x80)
            .lun(0)
            .command_len(0x0a)
            .build()
            .expect("Failed to build a read format capacities command block wrapper.");
        let data = CommandDataBlock::read_format_capacities();
        let mut wrapper = PageBox::from(CommandBlockWrapper::new(header, data));

        let (response, status): (PageBox<ReadFormatCapacities>, _) =
            self.send_scsi_command(&mut wrapper).await;

        status.check_corruption();
        Ok(*response)
    }

    async fn send_scsi_command<T>(
        &mut self,
        c: &mut PageBox<CommandBlockWrapper>,
    ) -> (PageBox<T>, PageBox<CommandStatusWrapper>)
    where
        T: Default,
    {
        self.send_command_block_wrapper(c).await;
        let response = self.receive_command_response().await;
        let status = self.receive_command_status().await;
        (response, status)
    }

    async fn send_command_block_wrapper(&mut self, c: &mut PageBox<CommandBlockWrapper>) {
        self.eps
            .issue_normal_trb(c, EndpointType::BulkOut)
            .await
            .expect("Failed to send a SCSI command.");
    }

    async fn receive_command_response<T>(&mut self) -> PageBox<T>
    where
        T: Default,
    {
        let c = PageBox::default();
        self.eps
            .issue_normal_trb(&c, EndpointType::BulkIn)
            .await
            .expect("Failed to receive a SCSI command reponse.");
        c
    }

    async fn receive_command_status(&mut self) -> PageBox<CommandStatusWrapper> {
        let b = PageBox::default();
        self.eps
            .issue_normal_trb(&b, EndpointType::BulkIn)
            .await
            .expect("Failed to receive a SCSI status.");
        b
    }
}
