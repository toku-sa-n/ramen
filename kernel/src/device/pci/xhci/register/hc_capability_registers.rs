// SPDX-License-Identifier: GPL-3.0-or-later

use {crate::mem::accessor::Accessor, bitfield::bitfield, os_units::Bytes, x86_64::PhysAddr};

pub struct HCCapabilityRegisters {
    cap_length: Accessor<CapabilityRegistersLength>,
    hcs_params_1: Accessor<StructuralParameters1>,
    hcs_params_2: Accessor<StructuralParameters2>,
    hc_cp_params_1: Accessor<HCCapabilityParameters1>,
    db_off: Accessor<DoorbellOffset>,
    rts_off: Accessor<RuntimeRegisterSpaceOffset>,
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

    pub fn offset_to_runtime_registers(&self) -> u32 {
        info!("RTSOFF: {}", self.rts_off.get());
        self.rts_off.get()
    }

    pub fn db_off(&self) -> &DoorbellOffset {
        &*self.db_off
    }

    pub fn max_num_of_erst(&self) -> MaxNumOfErst {
        MaxNumOfErst::new(2_u16.pow(self.hcs_params_2.erst_max()))
    }

    pub fn max_num_of_ports(&self) -> MaxNumOfPorts {
        self.hcs_params_1.max_num_of_ports()
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
    struct StructuralParameters1(u32);
    u8, max_slots, _: 7, 0;
    u8, max_ports, _: 31, 24;
}
impl StructuralParameters1 {
    fn number_of_device_slots(&self) -> NumberOfDeviceSlots {
        NumberOfDeviceSlots::new(self.max_slots())
    }

    fn max_num_of_ports(&self) -> MaxNumOfPorts {
        MaxNumOfPorts::new(self.max_ports())
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

pub struct MaxNumOfPorts(u8);
impl MaxNumOfPorts {
    fn new(num: u8) -> Self {
        Self(num)
    }
}
impl From<MaxNumOfPorts> for u8 {
    fn from(max: MaxNumOfPorts) -> Self {
        max.0
    }
}

bitfield! {
    #[repr(transparent)]
    struct StructuralParameters2(u32);
    erst_max, _: 7, 4;
}

#[derive(Copy, Clone)]
pub struct MaxNumOfErst(u16);
impl MaxNumOfErst {
    fn new(num: u16) -> Self {
        Self(num)
    }
}
impl From<MaxNumOfErst> for u16 {
    fn from(erst_max: MaxNumOfErst) -> Self {
        erst_max.0
    }
}
impl From<MaxNumOfErst> for usize {
    fn from(erst_max: MaxNumOfErst) -> Self {
        usize::from(erst_max.0)
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
struct RuntimeRegisterSpaceOffset(u32);
impl RuntimeRegisterSpaceOffset {
    fn get(&self) -> u32 {
        self.0 & !0x1f
    }
}
