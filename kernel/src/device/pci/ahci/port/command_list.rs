// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::Registers, crate::mem::allocator::page_box::PageBox, bitfield::bitfield,
    core::convert::TryInto,
};

pub struct CommandList(PageBox<[CommandHeader]>);
impl CommandList {
    pub fn new(registers: &Registers) -> Self {
        Self(PageBox::new_slice(
            CommandHeader::null(),
            Self::num_of_command_slots_supported(registers)
                .try_into()
                .unwrap(),
        ))
    }

    fn num_of_command_slots_supported(registers: &Registers) -> u32 {
        registers.generic.cap.read().num_of_command_slots()
    }
}

#[derive(Copy, Clone)]
pub struct CommandHeader(CommandHeaderStructure<[u32; 8]>);
impl CommandHeader {
    fn null() -> Self {
        Self(CommandHeaderStructure::null())
    }
}
bitfield! {
    #[repr(transparent)]
    #[derive(Copy,Clone)]
    pub struct CommandHeaderStructure([u32]);
    impl Debug;
}
impl CommandHeaderStructure<[u32; 8]> {
    fn null() -> Self {
        Self([0; 8])
    }
}
