// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{MessageAddress, MessageData, RegisterIndex, Registers},
    bitfield::bitfield,
};

#[derive(Debug)]
pub struct CapabilitySpec<'a> {
    registers: &'a Registers,
    base: RegisterIndex,
}

impl<'a> CapabilitySpec<'a> {
    pub fn new(registers: &'a Registers, base: RegisterIndex) -> Self {
        Self { registers, base }
    }

    fn edit_message_address<T>(&self, f: T)
    where
        T: Fn(&mut MessageAddress),
    {
        let mut message_address = self.get_message_address();
        f(&mut message_address);
        self.set_message_address(message_address);
    }

    fn get_message_address(&self) -> MessageAddress {
        MessageAddress::from(self.registers.get(self.base + 1))
    }

    fn set_message_address(&self, message_address: MessageAddress) {
        self.registers.set(self.base + 1, message_address.into())
    }

    fn edit_message_data<T>(&self, f: T)
    where
        T: Fn(&mut MessageData),
    {
        let mut message_data = self.get_message_data();
        f(&mut message_data);
        self.set_message_data(message_data);
    }

    fn get_message_data(&self) -> MessageData {
        MessageData::from(self.registers.get(self.base + 3))
    }

    fn set_message_data(&self, message_data: MessageData) {
        self.registers.set(self.base + 3, message_data.into())
    }

    fn get_message_control(&self) -> MessageControl {
        MessageControl::from((self.registers.get(self.base) >> 16) as u16 & 0xffff)
    }
}

bitfield! {
    struct MessageControl(u16);

    interrupt_status, set_interrupt_status: 16;
}

impl From<u16> for MessageControl {
    fn from(control: u16) -> Self {
        Self(control)
    }
}
