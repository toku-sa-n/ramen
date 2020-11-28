// SPDX-License-Identifier: GPL-3.0-or-later

use core::ptr;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(Debug)]
pub enum Descriptor {
    Device(Device),
    Configuration,
    Str,
    Interface(Interface),
    Endpoint,
    Hid,
}
impl Descriptor {
    pub fn from_slice(raw: &[u8]) -> Result<Self, Error> {
        assert_eq!(raw.len(), raw[0].into());
        match FromPrimitive::from_u8(raw[1]) {
            Some(t) => match t {
                // Safety: This operation is safe because the length of `raw` is equivalent to the
                // one of the descriptor.
                Ty::Device => Ok(Self::Device(unsafe {
                    ptr::read(raw as *const [u8] as *const _)
                })),
                Ty::Configuration => Ok(Self::Configuration),
                Ty::Str => Ok(Self::Str),
                Ty::Interface => Ok(Self::Interface(unsafe {
                    ptr::read(raw as *const [u8] as *const _)
                })),
                Ty::Endpoint => Ok(Self::Endpoint),
                Ty::Hid => Ok(Self::Hid),
            },
            None => Err(Error::UnrecognizedType(raw[1])),
        }
    }
}

#[derive(Copy, Clone, Default, Debug)]
#[repr(C, packed)]
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

#[derive(Copy, Clone, Default, Debug)]
#[repr(C, packed)]
pub struct Interface {
    len: u8,
    descriptor_type: u8,
    interface_number: u8,
    alternate_setting: u8,
    num_endpoints: u8,
    interface_class: u8,
    interface_subclass: u8,
    interface_protocol: u8,
    interface: u8,
}

#[derive(FromPrimitive)]
pub enum Ty {
    Device = 1,
    Configuration = 2,
    Str = 3,
    Interface = 4,
    Endpoint = 5,
    Hid = 33,
}

#[derive(Debug)]
pub enum Error {
    UnrecognizedType(u8),
}
