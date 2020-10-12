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

    pub fn reset_hc(&mut self) {
        if self.usb_sts.hc_halted() {
            return;
        }
        self.usb_cmd.reset();
    }
}

bitfield! {
    #[repr(transparent)]
    #[derive(Copy,Clone)]
    pub struct UsbCommandRegister(u32);

    pub run_stop,set_run_stop: 0;
    hc_reset,set_hc_reset: 1;
    pub interrupt_enable,set_interrupt_enable: 2;
}
impl UsbCommandRegister {
    fn reset(&mut self) {
        self.set_hc_reset(true);
        self.wait_until_hc_is_reset();
    }

    fn wait_until_hc_is_reset(&self) {
        while self.hc_reset() {}
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct UsbStatusRegister(u32);

    hc_halted, _: 0;
    pub controller_not_ready,_:11;
}

bitfield! {
    #[repr(transparent)]
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
    #[repr(transparent)]
    pub struct ConfigureRegister(u32);

    pub max_device_slots_enabled,set_max_device_slots_enabled:7,0;
}

bitfield! {
    #[repr(transparent)]
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
    #[repr(transparent)]
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
