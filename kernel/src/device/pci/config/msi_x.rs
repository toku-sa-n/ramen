// SPDX-License-Identifier: GPL-3.0-or-later

use {
    bitfield::bitfield,
    core::{
        marker::PhantomData,
        mem::size_of,
        ops::{Index, IndexMut},
    },
    x86_64::VirtAddr,
};

bitfield! {
    pub struct MsiX([u8]);
    u32;
    capability_id, _: 7, 0;
    table_size, _: 25, 16;
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
