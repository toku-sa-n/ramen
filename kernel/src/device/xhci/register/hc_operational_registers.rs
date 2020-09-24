// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{
        accessor::Accessor,
        device::xhci::register::hc_capability_registers::CapabilityRegistersLength,
    },
    bitfield::bitfield,
    x86_64::PhysAddr,
};

pub struct HCOperationalRegisters<'a> {
    pub usb_sts: Accessor<'a, UsbStatusRegister>,
    pub crcr: Accessor<'a, CommandRingControlRegister>,
    pub dcbaap: Accessor<'a, DeviceContextBaseAddressArrayPointer>,
    pub config: Accessor<'a, ConfigureRegister>,
}

impl<'a> HCOperationalRegisters<'a> {
    pub fn new(mmio_base: PhysAddr, cap_length: &mut CapabilityRegistersLength) -> Self {
        let operational_base = mmio_base + cap_length.len();

        let usb_sts = Accessor::new(operational_base, 0x04);
        let crcr = Accessor::new(operational_base, 0x18);
        let dcbaap = Accessor::new(operational_base, 0x30);
        let config = Accessor::new(operational_base, 0x38);

        Self {
            usb_sts,
            crcr,
            dcbaap,
            config,
        }
    }
}

bitfield! {
    pub struct UsbStatusRegister(u32);

    pub controller_not_ready,_:11;
}

bitfield! {
    pub struct CommandRingControlRegister(u64);

    ptr,set_pointer:63,6;
}

impl CommandRingControlRegister {
    pub fn set_ptr(&mut self, ptr: PhysAddr) {
        let ptr = ptr.as_u64() >> 6;

        self.set_pointer(ptr);
    }
}

bitfield! {
    pub struct ConfigureRegister(u32);

    pub max_device_slots_enabled,set_max_device_slots_enabled:7,0;
}

bitfield! {
    pub struct DeviceContextBaseAddressArrayPointer(u64);

    ptr,set_pointer:63,6;
}

impl DeviceContextBaseAddressArrayPointer {
    pub fn set_ptr(&mut self, ptr: PhysAddr) {
        let ptr = ptr.as_u64() >> 6;

        self.set_pointer(ptr);
    }
}
