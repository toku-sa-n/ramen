// SPDX-License-Identifier: GPL-3.0-or-later

mod scsi;

use crate::device::pci::xhci::port::fully_operational::FullyOperational;
use page_box::PageBox;
use scsi::{
    response::{Inquiry, Read10, ReadCapacity},
    CommandBlockWrapper, CommandBlockWrapperHeaderBuilder, CommandDataBlock, CommandStatusWrapper,
};
use xhci::context::EndpointType;

pub(in crate::device::pci::xhci::port) async fn task(eps: FullyOperational) {
    let mut m = MassStorage::new(eps);
    info!("This is the task of USB Mass Storage.");
    let b = m.inquiry().await;
    info!("Inquiry Command: {:?}", b);

    let b = m.read_capacity().await;
    info!("Read Capacity: {:?}", b);

    let b = m.read10().await;
    info!("Buf: {:X?}", b);
}

struct MassStorage {
    eps: FullyOperational,
}
impl MassStorage {
    fn new(eps: FullyOperational) -> Self {
        Self { eps }
    }

    async fn inquiry(&mut self) -> Inquiry {
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
        *response
    }

    async fn read_capacity(&mut self) -> ReadCapacity {
        let header = CommandBlockWrapperHeaderBuilder::default()
            .transfer_length(8)
            .flags(0x80)
            .lun(0)
            .command_len(10)
            .build()
            .expect("Failed to build a read capacity command block wrapper");
        let data = CommandDataBlock::read_capacity();
        let mut wrapper = PageBox::from(CommandBlockWrapper::new(header, data));

        let (response, status): (PageBox<ReadCapacity>, _) =
            self.send_scsi_command(&mut wrapper).await;

        status.check_corruption();
        *response
    }

    async fn read10(&mut self) -> PageBox<Read10> {
        let header = CommandBlockWrapperHeaderBuilder::default()
            .transfer_length(0x8000)
            .flags(0x80)
            .lun(0)
            .command_len(0x0a)
            .build()
            .expect("Failed to build a read 10 command block wrapper.");
        let data = CommandDataBlock::read10();
        let mut wrapper = PageBox::from(CommandBlockWrapper::new(header, data));

        let (response, status): (PageBox<Read10>, _) = self.send_scsi_command(&mut wrapper).await;

        status.check_corruption();
        response
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
