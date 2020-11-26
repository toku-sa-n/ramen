// SPDX-License-Identifier: GPL-3.0-or-later

// Temporary implementation.
#[derive(Default, Debug)]
pub struct Device {
    len: u8,
    descriptor_type: u8,
    cd_usb: u16,
    class: u8,
    subclass: u8,
    protocol: u8,
    max_packet_size0: u8,
    vendor: u16,
    product_id: u16,
    device: u16,
    manufacture: u8,
    product: u8,
    serial_number: u8,
    num_configurations: u8,
}
impl Device {
    pub fn class(&self) -> u8 {
        self.class
    }

    pub fn subclass(&self) -> u8 {
        self.subclass
    }

    pub fn protocol(&self) -> u8 {
        self.protocol
    }
}
