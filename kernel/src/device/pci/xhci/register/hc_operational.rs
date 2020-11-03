// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::hc_capability_registers::{HCCapabilityRegisters, NumberOfDeviceSlots},
    crate::mem::accessor::Accessor,
    bitfield::bitfield,
    os_units::Bytes,
    x86_64::PhysAddr,
};

pub struct HCOperational {
    pub usb_cmd: Accessor<UsbCommandRegister>,
    pub usb_sts: Accessor<UsbStatusRegister>,
    crcr: Accessor<CommandRingControlRegister>,
    dcbaap: Accessor<DeviceContextBaseAddressArrayPointer>,
    config: Accessor<ConfigureRegister>,
}

impl HCOperational {
    pub fn new(mmio_base: PhysAddr, capabilities: &HCCapabilityRegisters) -> Self {
        let operational_base = mmio_base + capabilities.len();

        let usb_cmd = Accessor::new(operational_base, Bytes::new(0x00));
        let usb_sts = Accessor::new(operational_base, Bytes::new(0x04));
        let crcr = Accessor::new(operational_base, Bytes::new(0x18));
        let dcbaap = Accessor::new(operational_base, Bytes::new(0x30));
        let config = Accessor::new(operational_base, Bytes::new(0x38));

        Self {
            usb_cmd,
            usb_sts,
            crcr,
            dcbaap,
            config,
        }
    }

    pub fn wait_until_hc_is_ready(&self) {
        self.usb_sts.wait_until_hc_is_ready();
    }

    pub fn set_num_of_device_slots(&mut self, num: NumberOfDeviceSlots) {
        self.config.set_num_of_slots(num)
    }

    pub fn set_dcbaa_ptr(&mut self, addr: PhysAddr) {
        self.dcbaap.set_ptr(addr)
    }

    pub fn set_command_ring_ptr(&mut self, addr: PhysAddr) {
        self.crcr.set_ptr(addr)
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct UsbCommandRegister(u32);

    pub _ ,set_run_stop: 0;
    pub hc_reset,set_hc_reset: 1;
    interrupt_enable,set_interrupt_enable: 2;
}

bitfield! {
    #[repr(transparent)]
    pub struct UsbStatusRegister(u32);

    pub hc_halted, _: 0;
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

     u8, _ ,set_max_device_slots_enabled:7,0;
}
impl ConfigureRegister {
    fn set_num_of_slots(&mut self, num: NumberOfDeviceSlots) {
        self.set_max_device_slots_enabled(num.into())
    }
}

bitfield! {
    #[repr(transparent)]
     struct PortStatusAndControlRegister(u32);

     current_connect_status, _: 0;
     port_enabled_disabled, _: 1;
     port_reset, _: 4;
     port_power, _: 9;
}

impl PortStatusAndControlRegister {}
