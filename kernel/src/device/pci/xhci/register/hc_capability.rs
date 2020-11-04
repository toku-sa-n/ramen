// SPDX-License-Identifier: GPL-3.0-or-later

use {crate::mem::accessor::Accessor, bitfield::bitfield, os_units::Bytes, x86_64::PhysAddr};

pub struct HCCapabilityRegisters {
    cap_length: Accessor<CapabilityRegistersLength>,
    pub hcs_params_1: Accessor<StructuralParameters1>,
    pub hcs_params_2: Accessor<StructuralParameters2>,
    hc_cp_params_1: Accessor<HCCapabilityParameters1>,
    pub db_off: Accessor<DoorbellOffset>,
    pub rts_off: Accessor<RuntimeRegisterSpaceOffset>,
}

impl HCCapabilityRegisters {
    pub fn new(mmio_base: PhysAddr) -> Self {
        let cap_length = Accessor::new(mmio_base, Bytes::new(0));
        let hcs_params_1 = Accessor::new(mmio_base, Bytes::new(0x04));
        let hcs_params_2 = Accessor::new(mmio_base, Bytes::new(0x08));
        let hc_cp_params_1 = Accessor::new(mmio_base, Bytes::new(0x10));
        let db_off = Accessor::new(mmio_base, Bytes::new(0x14));
        let rts_off = Accessor::new(mmio_base, Bytes::new(0x18));

        let hci_version = Accessor::<HCInterfaceVersionNumber>::new(mmio_base, Bytes::new(0x2));
        info!("xHC version: {:X}", hci_version.get());

        Self {
            cap_length,
            hcs_params_1,
            hcs_params_2,
            hc_cp_params_1,
            db_off,
            rts_off,
        }
    }

    pub fn number_of_device_slots(&self) -> NumberOfDeviceSlots {
        self.hcs_params_1.number_of_device_slots()
    }

    pub fn xhci_capability_ptr(&self) -> u32 {
        self.hc_cp_params_1.xhci_extended_capabilities_pointer()
    }

    pub fn len(&self) -> usize {
        self.cap_length.get()
    }
}

#[repr(transparent)]
struct CapabilityRegistersLength(u8);

impl CapabilityRegistersLength {
    fn get(&self) -> usize {
        self.0 as _
    }
}

#[repr(transparent)]
struct HCInterfaceVersionNumber(u16);
impl HCInterfaceVersionNumber {
    fn get(&self) -> u16 {
        self.0
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct StructuralParameters1(u32);
    u8, max_slots, _: 7, 0;
    pub u8, max_ports, _: 31, 24;
}
impl StructuralParameters1 {
    fn number_of_device_slots(&self) -> NumberOfDeviceSlots {
        NumberOfDeviceSlots::new(self.max_slots())
    }
}

pub struct NumberOfDeviceSlots(u8);
impl NumberOfDeviceSlots {
    fn new(num: u8) -> Self {
        Self(num)
    }
}
impl From<NumberOfDeviceSlots> for u8 {
    fn from(num: NumberOfDeviceSlots) -> Self {
        num.0
    }
}
impl From<NumberOfDeviceSlots> for usize {
    fn from(num: NumberOfDeviceSlots) -> Self {
        num.0.into()
    }
}

bitfield! {
    #[repr(transparent)]
    pub struct StructuralParameters2(u32);
    erst_max, _: 7, 4;
}
impl StructuralParameters2 {
    pub fn powered_erst_max(&self) -> u16 {
        2_u16.pow(self.erst_max())
    }
}

bitfield! {
    #[repr(transparent)]
    struct HCCapabilityParameters1(u32);
    xhci_extended_capabilities_pointer,_: 31,16;
}

#[repr(transparent)]
pub struct DoorbellOffset(u32);
impl DoorbellOffset {
    pub fn get(&self) -> u32 {
        self.0
    }
}

#[repr(transparent)]
pub struct RuntimeRegisterSpaceOffset(u32);
impl RuntimeRegisterSpaceOffset {
    pub fn get(&self) -> u32 {
        self.0 & !0x1f
    }
}
