// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{
        device::pci::xhci::register::hc_capability_registers::HCCapabilityRegisters,
        mem::accessor::{single_object, slice},
    },
    bitfield::bitfield,
    os_units::Size,
    x86_64::PhysAddr,
};

pub struct HCOperationalRegisters<'a> {
    usb_cmd: single_object::Accessor<'a, UsbCommandRegister>,
    usb_sts: single_object::Accessor<'a, UsbStatusRegister>,
    crcr: single_object::Accessor<'a, CommandRingControlRegister>,
    dcbaap: single_object::Accessor<'a, DeviceContextBaseAddressArrayPointer>,
    config: single_object::Accessor<'a, ConfigureRegister>,
    port_sc: slice::Accessor<'a, PortStatusAndControlRegister>,
}

impl<'a> HCOperationalRegisters<'a> {
    pub fn new(mmio_base: PhysAddr, capabilities: &HCCapabilityRegisters) -> Self {
        let operational_base = mmio_base + capabilities.len();

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

    pub fn wait_until_hc_is_ready(&self) {
        self.usb_sts.wait_until_hc_is_ready();
    }

    pub fn set_num_of_device_slots(&mut self, num: u32) {
        self.config.set_max_device_slots_enabled(num)
    }

    pub fn set_dcbaa_ptr(&mut self, addr: PhysAddr) {
        self.dcbaap.set_ptr(addr)
    }

    pub fn set_command_ring_ptr(&mut self, addr: PhysAddr) {
        self.crcr.set_ptr(addr)
    }

    pub fn enable_interrupt(&mut self) {
        self.usb_cmd.set_interrupt_enable(true)
    }

    pub fn run(&mut self) {
        self.usb_cmd.set_run_stop(true);
        while self.usb_sts.hc_halted() {}
    }
}

bitfield! {
    #[repr(transparent)]
    struct UsbCommandRegister(u32);

    run_stop,set_run_stop: 0;
    hc_reset,set_hc_reset: 1;
    interrupt_enable,set_interrupt_enable: 2;
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
    struct UsbStatusRegister(u32);

    hc_halted, _: 0;
    controller_not_ready,_:11;
}
impl UsbStatusRegister {
    fn wait_until_hc_is_ready(&self) {
        while self.controller_not_ready() {}
    }
}

bitfield! {
    #[repr(transparent)]
    struct CommandRingControlRegister(u64);

    ptr,set_pointer:63,6;
}

impl CommandRingControlRegister {
    fn set_ptr(&mut self, ptr: PhysAddr) {
        let ptr = ptr.as_u64() >> 6;

        self.set_pointer(ptr);
    }
}
#[repr(transparent)]
struct DeviceContextBaseAddressArrayPointer(u64);

impl DeviceContextBaseAddressArrayPointer {
    fn set_ptr(&mut self, ptr: PhysAddr) {
        assert!(
            ptr.as_u64().trailing_zeros() >= 6,
            "Wrong address: {:?}",
            ptr
        );

        self.0 = ptr.as_u64();
    }
}

bitfield! {
    #[repr(transparent)]
     struct ConfigureRegister(u32);

    max_device_slots_enabled,set_max_device_slots_enabled:7,0;
}

bitfield! {
    #[repr(transparent)]
     struct PortStatusAndControlRegister(u32);

     current_connect_status, _: 0;
     port_enabled_disabled, _: 1;
     port_reset, _: 4;
     port_power, _: 9;
}

impl PortStatusAndControlRegister {
    fn disconnected(&self) -> bool {
        self.port_power()
            && !self.current_connect_status()
            && !self.port_enabled_disabled()
            && !self.port_reset()
    }
}
