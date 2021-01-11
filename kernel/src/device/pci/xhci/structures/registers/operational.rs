// SPDX-License-Identifier: GPL-3.0-or-later

use super::capability::Capability;
use crate::mem::accessor::Accessor;
use bitfield::bitfield;
use os_units::Bytes;
use x86_64::PhysAddr;

pub struct Operational {
    pub usb_cmd: Accessor<UsbCommandRegister>,
    pub usb_sts: Accessor<UsbStatusRegister>,
    pub page_size: Accessor<PageSize>,
    pub crcr: Accessor<CommandRingControlRegister>,
    pub dcbaap: Accessor<DeviceContextBaseAddressArrayPointer>,
    pub config: Accessor<ConfigureRegister>,
    pub port_registers: Accessor<[PortRegisters]>,
}

impl Operational {
    /// SAFETY: This method is unsafe because if `mmio_base` is not a valid MMIO base address, it
    /// can violate memory safety.
    #[allow(clippy::too_many_lines)]
    pub unsafe fn new(mmio_base: PhysAddr, capabilities: &Capability) -> Self {
        let operational_base = mmio_base + capabilities.cap_length.read().get();

        macro_rules! accessor {
            ($bytes:expr) => {
                Accessor::user(operational_base, Bytes::new($bytes))
            };
        }

        let usb_cmd = accessor!(0x00);
        let usb_sts = accessor!(0x04);
        let page_size = accessor!(0x08);
        let crcr = accessor!(0x18);
        let dcbaap = accessor!(0x30);
        let config = accessor!(0x38);
        let port_registers = Accessor::user_slice(
            operational_base,
            Bytes::new(0x400),
            capabilities.hcs_params_1.read().max_ports().into(),
        );

        Self {
            usb_cmd,
            usb_sts,
            page_size,
            crcr,
            dcbaap,
            config,
            port_registers,
        }
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
    pub controller_not_ready,_:11;
    pub host_system_error, _: 2;
    pub hc_error, _: 12;
}

#[repr(transparent)]
pub struct PageSize(u32);
impl PageSize {
    pub fn as_bytes(&self) -> Bytes {
        Bytes::new(2_usize.pow(self.0 + 12))
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct CommandRingControlRegister(u64);
    impl Debug;
    pub _, set_ring_cycle_state: 0;
    command_ring_running, _: 3;
    _, set_pointer:63,6;
}
impl CommandRingControlRegister {
    pub fn set_ptr(&mut self, ptr: PhysAddr) {
        assert!(ptr.is_aligned(64_u64));
        assert!(!self.command_ring_running());
        let ptr = ptr.as_u64() >> 6;

        self.set_pointer(ptr);
    }
}

#[repr(transparent)]
pub struct DeviceContextBaseAddressArrayPointer(u64);
impl DeviceContextBaseAddressArrayPointer {
    pub fn set(&mut self, ptr: PhysAddr) {
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
     pub struct ConfigureRegister(u32);

     pub u8, _ ,set_max_device_slots_enabled:7,0;
}

#[derive(Debug)]
pub struct PortRegisters {
    pub port_sc: PortStatusAndControlRegister,
    _port_pmsc: u32,
    _port_li: u32,
    _port_hlpmc: u32,
}

bitfield! {
    #[repr(transparent)]
     pub  struct PortStatusAndControlRegister(u32);
     impl Debug;
     pub current_connect_status, _: 0;
     port_enabled_disabled, _: 1;
     pub port_reset, set_port_reset: 4;
     port_link_state, _: 8, 5;
     port_power, _: 9;
     pub port_speed, _: 13, 10;
     pub port_reset_changed, _: 21;
}
