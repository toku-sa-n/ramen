// SPDX-License-Identifier: GPL-3.0-or-later

use super::{MessageAddress, RegisterIndex, Registers};

#[derive(Debug)]
pub struct CapabilitySpec<'a> {
    registers: &'a Registers,
    base: RegisterIndex,
}

impl<'a> CapabilitySpec<'a> {
    pub fn new(registers: &'a Registers, base: RegisterIndex) -> Self {
        Self { registers, base }
    }

    fn get_message_address(&self) -> MessageAddress {
        MessageAddress::from(self.registers.get(self.base + 1))
    }

    fn set_message_address(&self, message_address: MessageAddress) {
        self.registers.set(self.base + 1, message_address.into())
    }
}
