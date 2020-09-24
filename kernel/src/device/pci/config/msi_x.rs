// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::device::pci::config::{Bus, ConfigAddress, Device, EndPoint, Function, Offset},
    alloc::vec::Vec,
    bitfield::bitfield,
    os_units::{Bytes, Size},
};

#[derive(Debug)]
pub struct MsiX(Vec<MsiXDescriptor>);
impl MsiX {
    pub fn new(bus: Bus, device: Device, capability_ptr: Offset, endpoint: &EndPoint) -> Self {
        let mut msi_x_collection = Vec::new();
        let mut next_ptr = capability_ptr;

        while {
            let descriptor = MsiXDescriptor::new(bus, device, next_ptr, endpoint);
            next_ptr = descriptor.next_ptr;
            msi_x_collection.push(descriptor);

            !next_ptr.is_null()
        } {}

        Self(msi_x_collection)
    }
}

#[derive(Debug)]
struct MsiXDescriptor {
    bir: Bir,
    table_offset: TableOffset,
    next_ptr: Offset,
}

impl MsiXDescriptor {
    fn new(bus: Bus, device: Device, base: Offset, endpoint: &EndPoint) -> Self {
        Self {
            bir: Bir::new(bus, device, base),
            table_offset: TableOffset::new(bus, device, base),
            next_ptr: fetch_next_ptr(bus, device, base),
        }
    }

    fn table(&mut self) -> &mut [Element] {
        unimplemented!()
    }
}

#[derive(Debug)]
struct Bir(u32);
impl Bir {
    fn new(bus: Bus, device: Device, capability_base: Offset) -> Self {
        let config_addr = ConfigAddress::new(bus, device, Function::zero(), capability_base + 0x04);
        let raw = unsafe { config_addr.read() };
        let bir = raw & 0b111;
        assert!(bir < 6);

        Self(bir)
    }
}

struct TableSize(u32);
impl TableSize {
    fn new(bus: Bus, device: Device, capability_base: Offset) -> Self {
        let config_addr = ConfigAddress::new(bus, device, Function::zero(), capability_base);
        let raw = unsafe { config_addr.read() };
        let size = (raw >> 16) & 0x7ff;

        Self(size)
    }
}

#[derive(Debug)]
struct TableOffset(Size<Bytes>);
impl TableOffset {
    fn new(bus: Bus, device: Device, capability_base: Offset) -> Self {
        let config_addr = ConfigAddress::new(bus, device, Function::zero(), capability_base);
        let raw = unsafe { config_addr.read() };
        let offset = Size::new((raw & !0b111) as _);

        Self(offset)
    }
}

fn fetch_next_ptr(bus: Bus, device: Device, capability_base: Offset) -> Offset {
    let config_addr = ConfigAddress::new(bus, device, Function::zero(), capability_base);
    let raw = unsafe { config_addr.read() };
    Offset::new((raw >> 8) & 0xff)
}

struct Element {
    message_address: MessageAddress,
    message_data: MessageData,
    vector_control: VectorControl,
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
