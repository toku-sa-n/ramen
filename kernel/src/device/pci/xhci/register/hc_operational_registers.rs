// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{
        accessor::{single_object, slice},
        device::pci::xhci::register::hc_capability_registers::CapabilityRegistersLength,
    },
    bitfield::bitfield,
    os_units::Size,
    x86_64::PhysAddr,
};

pub struct HCOperationalRegisters<'a> {
    pub usb_cmd: single_object::Accessor<'a, UsbCommandRegister>,
    pub usb_sts: single_object::Accessor<'a, UsbStatusRegister>,
    pub crcr: single_object::Accessor<'a, CommandRingControlRegister>,
    pub dcbaap: single_object::Accessor<'a, DeviceContextBaseAddressArrayPointer>,
    pub config: single_object::Accessor<'a, ConfigureRegister>,
    pub port_sc: slice::Accessor<'a, PortStatusAndControlRegister>,
}

impl<'a> HCOperationalRegisters<'a> {
    pub fn new(mmio_base: PhysAddr, cap_length: &CapabilityRegistersLength) -> Self {
        let operational_base = mmio_base + cap_length.get();

        let usb_cmd = single_object::Accessor::new(operational_base, 0x00);
        let usb_sts = single_object::Accessor::new(operational_base, 0x04);
        let crcr = single_object::Accessor::new(operational_base, 0x18);
        let dcbaap = single_object::Accessor::new(operational_base, 0x30);
        let config = single_object::Accessor::new(operational_base, 0x38);
        let port_sc = slice::Accessor::new(operational_base, Size::new(0x400), 10);

        Self {
            usb_cmd,
            usb_sts,
            crcr,
            dcbaap,
            config,
            port_sc,
        }
    }
}

bitfield! {
    pub struct UsbCommandRegister(u32);

    pub run_stop,set_run_stop: 0;
    pub interrupt_enable,set_interrupt_enable: 2;
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

bitfield! {
    pub struct PortStatusAndControlRegister(u32);

    pub current_connect_status, _: 0;
    pub port_enabled_disabled, _: 1;
    pub port_reset, _: 4;
    pub port_power, _: 9;
}

impl PortStatusAndControlRegister {
    pub fn disconnected(&self) -> bool {
        self.port_power()
            && !self.current_connect_status()
            && !self.port_enabled_disabled()
            && !self.port_reset()
    }
}
