// SPDX-License-Identifier: GPL-3.0-or-later

use super::{MessageAddress, MessageData, RegisterIndex, Registers};

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
}
