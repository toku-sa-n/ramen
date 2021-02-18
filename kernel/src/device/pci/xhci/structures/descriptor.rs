// SPDX-License-Identifier: GPL-3.0-or-later

use bit_field::BitField;
use core::{convert::TryInto, ptr};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use xhci::context::EndpointType;

#[derive(Copy, Clone, Debug)]
pub enum Descriptor {
    Device(Device),
    Configuration,
    Str,
    Interface(Interface),
    Endpoint(Endpoint),
    Hid,
}
impl Descriptor {
    pub fn from_slice(raw: &[u8]) -> Result<Self, Error> {
        assert_eq!(raw.len(), raw[0].into());
        match FromPrimitive::from_u8(raw[1]) {
            Some(t) => match t {
                // SAFETY: This operation is safe because the length of `raw` is equivalent to the
                // one of the descriptor.
                Ty::Device => Ok(Self::Device(unsafe {
                    ptr::read((raw as *const [u8]).cast())
                })),
                Ty::Configuration => Ok(Self::Configuration),
                Ty::Str => Ok(Self::Str),
                Ty::Interface => Ok(Self::Interface(unsafe {
                    ptr::read((raw as *const [u8]).cast())
                })),
                Ty::Endpoint => Ok(Self::Endpoint(unsafe {
                    ptr::read((raw as *const [u8]).cast())
                })),
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
impl Device {
    pub(in crate::device::pci::xhci) fn max_packet_size(&self) -> u16 {
        if let (3, _) = self.version() {
            2_u16.pow(self.max_packet_size0.into())
        } else {
            self.max_packet_size0.into()
        }
    }

    fn version(&self) -> (u8, u8) {
        let cd_usb = self.cd_usb;

        (
            (cd_usb >> 8).try_into().unwrap(),
            (cd_usb & 0xff).try_into().unwrap(),
        )
    }
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
impl Interface {
    pub fn ty(&self) -> (u8, u8, u8) {
        (
            self.interface_class,
            self.interface_subclass,
            self.interface_protocol,
        )
    }
}

#[derive(Copy, Clone, Default, Debug)]
#[repr(C, packed)]
pub struct Endpoint {
    len: u8,
    descriptor_type: u8,
    pub endpoint_address: u8,
    pub attributes: u8,
    pub max_packet_size: u16,
    pub interval: u8,
}
impl Endpoint {
    pub fn ty(self) -> EndpointType {
        EndpointType::from_u8(if self.attributes == 0 {
            4
        } else {
            self.attributes.get_bits(0..=1)
                + if self.endpoint_address.get_bit(7) {
                    4
                } else {
                    0
                }
        })
        .expect("EndpointType must be convertible from `attributes` and `endpoint_address`.")
    }

    pub fn doorbell_value(self) -> u32 {
        2 * u32::from(self.endpoint_address.get_bits(0..=3))
            + self.endpoint_address.get_bit(7) as u32
    }
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
