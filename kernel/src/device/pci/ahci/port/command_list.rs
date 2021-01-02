// SPDX-License-Identifier: GPL-3.0-or-later

use super::Registers;
use crate::mem::allocator::page_box::PageBox;
use alloc::vec::Vec;
use bitfield::bitfield;
use core::convert::TryInto;
use x86_64::PhysAddr;

const NUM_OF_PRDT: usize = 8;

pub struct CommandList {
    headers: PageBox<[Header]>,
    tables: Vec<PageBox<Table>>,
}
impl CommandList {
    pub fn new(registers: &Registers) -> Self {
        let headers = PageBox::user_slice(
            Header::null(),
            Self::num_of_command_slots_supported(registers),
        );
        let tables = Self::new_tables(registers);
        let mut list = Self { headers, tables };
        list.set_ptrs_of_headers();
        list
    }

    pub fn phys_addr_to_headers(&self) -> PhysAddr {
        self.headers.phys_addr()
    }

    fn new_tables(registers: &Registers) -> Vec<PageBox<Table>> {
        let mut tables = Vec::new();
        for _ in 0..Self::num_of_command_slots_supported(registers) {
            tables.push(PageBox::user(Table::null()));
        }
        tables
    }

    fn set_ptrs_of_headers(&mut self) {
        for header in self.headers.iter_mut() {
            header.set_command_table_base_addr(self.tables[0].phys_addr());
        }
    }

    fn num_of_command_slots_supported(registers: &Registers) -> usize {
        registers
            .generic
            .cap
            .read()
            .num_of_command_slots()
            .try_into()
            .unwrap()
    }
}

pub type Header = CommandHeaderStructure<[u32; 8]>;
bitfield! {
    #[repr(transparent)]
    #[derive(Copy,Clone)]
    pub struct CommandHeaderStructure([u32]);
    impl Debug;
    u64, _, set_command_table_base_addr_as_u64: 64+31, 64;
}
impl CommandHeaderStructure<[u32; 8]> {
    fn null() -> Self {
        Self([0; 8])
    }

    fn set_command_table_base_addr(&mut self, addr: PhysAddr) {
        assert!(addr.is_aligned(128_u64));
        self.set_command_table_base_addr_as_u64(addr.as_u64());
    }
}

pub struct Table {
    _rsvd: [u8; 0x80],
    _prdt: [PhysicalRegionDescriptorTable; NUM_OF_PRDT],
}
impl Table {
    fn null() -> Self {
        Self {
            _rsvd: [0; 0x80],
            _prdt: [PhysicalRegionDescriptorTable::null(); NUM_OF_PRDT],
        }
    }
}

pub type PhysicalRegionDescriptorTable = PhysicalRegionDescriptorTableStructure<[u32; 8]>;
bitfield! {
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct PhysicalRegionDescriptorTableStructure([u32]);
    impl Debug;
}
impl PhysicalRegionDescriptorTable {
    fn null() -> Self {
        Self([0; 8])
    }
}
