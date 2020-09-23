// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::device::pci::config::{Bus, CapabilityPtr, ConfigAddress},
    alloc::vec::Vec,
    bitfield::bitfield,
    core::{
        marker::PhantomData,
        mem::size_of,
        ops::{Index, IndexMut},
    },
    os_units::{Bytes, Size},
    x86_64::VirtAddr,
};

struct MsiX(Vec<MsiXDescriptor>);
impl MsiX {
    fn new(bus: u8, device: u8, capability_ptr: &CapabilityPtr) -> Self {
        unimplemented!();
    }
}

struct MsiXDescriptor {
    bir: usize,
    table_offset: Size<Bytes>,
}

impl MsiXDescriptor {
    fn new(bus: Bus, device: u8, offset_from_config_space_base: u8) -> Self {
        unimplemented!();
        let raw_data: [u32; 3];
        for i in 0..3 {
            let config_address =
                ConfigAddress::new(bus, device, 0, offset_from_config_space_base + i * 8);
            let row = unsafe { config_address.read() };
            raw_data[i as usize] = row;
        }

        assert_eq!(Self::capability_id(raw_data), 0x11);
        Self {
            bir: Self::bir(raw_data),
            table_offset: Self::table_offset(raw_data),
        }
    }

    fn capability_id(raw_data: [u32; 3]) -> u8 {
        (raw_data[0] & 0xff) as u8
    }

    fn bir(raw_data: [u32; 3]) -> usize {
        (raw_data[1] & 0b111) as usize
    }

    fn table_offset(raw_data: [u32; 3]) -> Size<Bytes> {
        Size::new((raw_data[1] & !0b111) as usize)
    }
}

struct Table<'a> {
    base: VirtAddr,
    num: usize,
    _marker: PhantomData<&'a Element>,
}

impl<'a> Index<usize> for Table<'a> {
    type Output = Element;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.num);
        unsafe { &*((self.base.as_u64() as usize + size_of::<Element>() * index) as *const _) }
    }
}

impl<'a> IndexMut<usize> for Table<'a> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.num);
        unsafe { &mut *((self.base.as_u64() as usize + size_of::<Element>() * index) as *mut _) }
    }
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
